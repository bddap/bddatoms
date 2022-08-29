use core::mem::size_of;
use std::{marker::PhantomData, sync::Arc};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferBinding,
};

/// A *typed* buffer on the gpu
pub struct GpuBuf<T> {
    inner: wgpu::Buffer,

    // current number of elements
    length: usize,
    capacity: usize,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    _spook: PhantomData<*const T>,
}

impl<T> GpuBuf<T> {
    /// create and set contents
    pub fn initialize(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        content: &[T],
        usage: wgpu::BufferUsages,
    ) -> Self
    where
        T: bytemuck::NoUninit,
    {
        Self {
            inner: device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(content),
                usage,
            }),
            length: content.len(),
            capacity: content.len(),
            device,
            queue,
            _spook: PhantomData,
        }
    }

    // Maps the buffer to to host memory, writes, then unmaps.
    // This is all done synchronously. Perhaps not the fastest solution,
    // but simple.
    pub fn copy_from_slice(&mut self, target: &[T])
    where
        T: bytemuck::NoUninit,
    {
        assert!(target.len() <= self.capacity);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let as_raw: &[u8] = bytemuck::cast_slice(target);
        let nu = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: as_raw,
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        self.length = target.len();
        encoder.copy_buffer_to_buffer(&nu, 0, &self.inner, 0, as_raw.len() as u64);
        self.queue.submit([encoder.finish()]);
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_entire_buffer_binding(&self) -> BufferBinding {
        self.inner.as_entire_buffer_binding()
    }
}
