# Tools
ASM := rgbasm
LINK := rgblink
FIX := rgbfix

# Target
ROM := serial-test

# Directories
SRCDIR := src
INCDIR := include
BUILDDIR := build

# File extensions
SRCEXT := asm
OBJEXT := o

# Flags
ASM_FLAGS := -i $(INCDIR)
LINK_FLAGS :=
FIX_FLAGS := --fix-spec hg \
			 --title "SERIAL TEST" \
			 --mbc-type 0 \
			 --ram-size 0 \
			 --pad-value 0xFF

# Files
SOURCES := $(SRCDIR)/main.asm $(SRCDIR)/sio.asm
OBJECTS := $(patsubst $(SRCDIR)/%,$(BUILDDIR)/%,$(SOURCES:.$(SRCEXT)=.$(OBJEXT)))

# Targets
.PHONY: all clean

all: $(ROM)

$(ROM): $(OBJECTS)
	$(LINK) -o $(BUILDDIR)/$@.gb -n $(BUILDDIR)/$@.sym $(OBJECTS)
	$(FIX) $(FIX_FLAGS) $(BUILDDIR)/$@.gb

$(BUILDDIR)/%.$(OBJEXT): $(SRCDIR)/%.$(SRCEXT)
	mkdir -p $(dir $@)
	$(ASM) $(ASM_FLAGS) -o $@ $<

clean:
	rm -rf $(BUILDDIR)
