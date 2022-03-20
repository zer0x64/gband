ASM = rgbasm
LINK = rgblink
FIX = rgbfix

ROM_NAME = vuln_rom
SOURCES = src/main.asm
FIX_FLAGS = -v -p 0

INCDIR = inc
BUILD_DIR = build
OBJECTS = $(SOURCES:src/%.asm=%.o)

all: $(ROM_NAME).gb

$(ROM_NAME).gb: $(BUILD_DIR)/$(OBJECTS) $(BUILD_DIR)
	$(LINK) -o $(ROM_NAME).gb -n $(ROM_NAME).sym $(BUILD_DIR)/$(OBJECTS)
	$(FIX) $(FIX_FLAGS) $@

$(BUILD_DIR)/%.o: src/%.asm $(BUILD_DIR)
	$(ASM) -i $(INCDIR)/ -o $@ $<

$(BUILD_DIR): 
	mkdir $(BUILD_DIR)

clean:
	rm -rf $(BUILD_DIR) $(ROM_NAME).gb $(ROM_NAME).sym