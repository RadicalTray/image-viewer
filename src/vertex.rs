use ash::vk;

pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        let bind_desc = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>().try_into().unwrap())
            .input_rate(vk::VertexInputRate::VERTEX);

        [bind_desc]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos_attribute = vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(std::mem::offset_of!(Vertex, pos).try_into().unwrap());
        let color_attribute = vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(std::mem::offset_of!(Vertex, color).try_into().unwrap());

        [pos_attribute, color_attribute]
    }
}
