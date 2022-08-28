use core::mem::size_of;
use std::{marker::PhantomData, ops::DerefMut, sync::Arc};

/// A *typed* buffer on the gpu
pub struct GpuBuf<T> {
    inner: wgpu::Buffer,
    size_octets: usize,

    // current number of elements
    length: usize,

    usage: wgpu::BufferUsages,
    device: Arc<wgpu::Device>,
    _spook: PhantomData<*const T>,
}

impl<T> GpuBuf<T> {
    pub fn create(
        device: Arc<wgpu::Device>,
        initial_capacity: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let size_octets = initial_capacity * size_of::<T>();
        Self {
            inner: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size_octets as u64,
                usage,
                mapped_at_creation: true,
            }),
            size_octets,
            length: 0,
            usage,
            device,
            _spook: PhantomData,
        }
    }


    // Maps the buffer to to host memory, writes, then unmaps.
    // This is all done synchronously. Perhaps not the fastest solution,
    // but simple.
    pub fn set_from_slice(&mut self, target: &[T])
    where
        T: bytemuck::NoUninit,
    {
        let target_size = target.len() * size_of::<T>();
        if self.size_octets < target_size {
            self.size_octets = target_size;
            self.inner = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: target_size as u64,
                usage: self.usage,
                mapped_at_creation: true,
            });
        }
        self.inner.slice(..).get_mapped_range_mut().deref_mut()[0..target_size]
            .copy_from_slice(bytemuck::cast_slice(target));

        self.length = target.len();
        self.inner.unmap();
    }

    pub fn len(&self) -> usize {
        self.length
    }

    // returns None if the slice would be empty
    // this is because wgpu does not allow empty slices
    pub fn slice(&self) -> Option<wgpu::BufferSlice> {
        if self.is_empty() {
            None
        } else {
            let end = self.len() * size_of::<T>();
            Some(self.inner.slice(0..(end as u64)))
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
