BUILD_DIR := build

SHADERS_DIR := shaders
SHADERS := $(shell find $(SHADERS_DIR) -name 'shader.*')
TARGET_SHADERS := $(SHADERS:$(SHADERS_DIR)/shader.%=$(BUILD_DIR)/shaders/%.spv)

.PHONY: all
all: shaders

.PHONY: shaders
shaders: $(TARGET_SHADERS)

$(BUILD_DIR)/$(SHADERS_DIR)/%.spv: $(SHADERS_DIR)/shader.%
	mkdir -p $(dir $@)
	glslc $< -o $@

.PHONY: clean
clean:
	rm -r $(BUILD_DIR)
