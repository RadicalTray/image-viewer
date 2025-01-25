use ash::vk;

pub struct Vertex {
    pub pos: glam::Vec2,
    pub color: glam::Vec3,
}

impl Vertex {
    pub fn get_binding_descriptions()
    -> Result<[vk::VertexInputBindingDescription; 1], <u32 as TryFrom<usize>>::Error> {
        let bind_desc = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>().try_into()?)
            .input_rate(vk::VertexInputRate::VERTEX);

        Ok([bind_desc])
    }

    pub fn get_attribute_descriptions()
    -> Result<[vk::VertexInputAttributeDescription; 2], <u32 as TryFrom<usize>>::Error> {
        let pos_attribute = vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(std::mem::offset_of!(Vertex, pos).try_into()?);
        let color_attribute = vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(std::mem::offset_of!(Vertex, color).try_into()?);

        Ok([pos_attribute, color_attribute])
    }
}
