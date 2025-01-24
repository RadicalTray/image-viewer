use crate::device::Device;
use ash::prelude::VkResult;
use ash::vk;
use std::error::Error;
use std::ffi::c_void;
use std::rc::Rc;

#[derive(Debug)]
pub enum BufferCreationError {
    MemoryTypeNotFound,
}

pub struct Buffer<'a> {
    ash_device: Rc<Device<'a>>,
    size: vk::DeviceSize,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    ptr: Option<*mut c_void>,
    allocator: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> Buffer<'a> {
    pub fn new(
        ash_device: Rc<Device<'a>>,
        buffer_info: &vk::BufferCreateInfo,
        allocator: Option<&'a vk::AllocationCallbacks<'a>>,
        mem_props: vk::MemoryPropertyFlags,
        device_mem_props: vk::PhysicalDeviceMemoryProperties,
    ) -> Result<Self, Box<dyn Error>> {
        let size = buffer_info.size;
        let buffer = unsafe { ash_device.device().create_buffer(buffer_info, allocator)? };
        let mem_requirements =
            unsafe { ash_device.device().get_buffer_memory_requirements(buffer) };

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(
                Self::find_memory_type_index(
                    device_mem_props,
                    mem_requirements.memory_type_bits,
                    mem_props,
                )?
                .try_into()?,
            );

        let memory = unsafe {
            ash_device
                .device()
                .allocate_memory(&alloc_info, allocator)?
        };
        unsafe { ash_device.device().bind_buffer_memory(buffer, memory, 0)? };

        Ok(Self {
            ash_device,
            size,
            buffer,
            memory,
            ptr: None,
            allocator,
        })
    }

    pub fn find_memory_type_index(
        device_mem_props: vk::PhysicalDeviceMemoryProperties,
        type_filter: u32,
        mem_props: vk::MemoryPropertyFlags,
    ) -> Result<usize, BufferCreationError> {
        for (i, mem_type) in device_mem_props.memory_types.iter().enumerate() {
            if ((type_filter & (1 << i)) != 0)
                && ((mem_type.property_flags & mem_props) == mem_props)
            {
                return Ok(i);
            }
        }

        Err(BufferCreationError::MemoryTypeNotFound)
    }

    pub fn map_memory(
        &mut self,
        offset: vk::DeviceSize,
        flags: vk::MemoryMapFlags,
    ) -> VkResult<()> {
        self.ptr = unsafe {
            Some(
                self.ash_device
                    .device()
                    .map_memory(self.memory(), offset, self.size, flags)?,
            )
        };
        Ok(())
    }

    pub fn unmap_memory(&mut self) {
        self.ptr = None;
        unsafe {
            self.ash_device.device().unmap_memory(self.memory());
        }
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn ptr(&self) -> Option<*mut c_void> {
        self.ptr
    }

    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.ash_device
                .device()
                .free_memory(self.memory, self.allocator);
            self.ash_device
                .device()
                .destroy_buffer(self.buffer, self.allocator);
        }
    }
}

impl std::fmt::Display for BufferCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferCreationError::MemoryTypeNotFound => {
                write!(f, "Failed to find suitable memory type!")
            }
        }
    }
}

impl Error for BufferCreationError {}
