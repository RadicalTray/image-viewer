use ash::prelude::VkResult;
use ash::vk;
use std::error::Error;
use std::ffi::c_void;

#[derive(Debug)]
pub enum BufferCreationError {
    MemoryTypeNotFound,
}

#[derive(Debug)]
pub struct Buffer {
    size: vk::DeviceSize,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    ptr: Option<*mut c_void>,
}

impl Buffer {
    pub fn new(
        device: &ash::Device,
        buffer_info: &vk::BufferCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks>,
        mem_props: vk::MemoryPropertyFlags,
        device_mem_props: vk::PhysicalDeviceMemoryProperties,
    ) -> Result<Self, Box<dyn Error>> {
        let size = buffer_info.size;
        let buffer = unsafe { device.create_buffer(buffer_info, allocation_callbacks)? };
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

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

        let memory = unsafe { device.allocate_memory(&alloc_info, allocation_callbacks)? };
        unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

        Ok(Self {
            size,
            buffer,
            memory,
            ptr: None,
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
        device: &ash::Device,
        offset: vk::DeviceSize,
        flags: vk::MemoryMapFlags,
    ) -> VkResult<()> {
        self.ptr = unsafe { Some(device.map_memory(self.memory(), offset, self.size, flags)?) };
        Ok(())
    }

    pub fn unmap_memory(&mut self, device: &ash::Device) {
        self.ptr = None;
        unsafe {
            device.unmap_memory(self.memory());
        }
    }

    pub fn cleanup(
        self,
        device: &ash::Device,
        allocation_callbacks: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            device.destroy_buffer(self.buffer(), allocation_callbacks);
            device.free_memory(self.memory(), allocation_callbacks);
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
