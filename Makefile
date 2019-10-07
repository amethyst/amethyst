# This Makefile is meant for shader compilation only.
# Use cargo to compile the rust part of the project.

GLSLC = $(shell ./find_glslc.sh)
ifeq "$(GLSLC)" ""
	break;
endif

FLAGS = -c -g

SHADERS=$(wildcard amethyst_rendy/shaders/**/*)
COMP_SHADERS = $(patsubst amethyst_rendy/shaders/%,amethyst_rendy/compiled/%.spv,$(SHADERS))
COMP_DISASMS = $(patsubst amethyst_rendy/shaders/%,amethyst_rendy/compiled/%.spvasm,$(SHADERS))
SHADERS_UI=$(wildcard amethyst_ui/shaders/*)
COMP_SHADERS_UI = $(patsubst amethyst_ui/shaders/%,amethyst_ui/compiled/%.spv,$(SHADERS_UI))
COMP_DISASMS_UI = $(patsubst amethyst_ui/shaders/%,amethyst_ui/compiled/%.spvasm,$(SHADERS_UI))
SHADERS_TILES=$(wildcard amethyst_tiles/shaders/*)
COMP_SHADERS_TILES = $(patsubst amethyst_tiles/shaders/%,amethyst_tiles/compiled/%.spv,$(SHADERS_TILES))
COMP_DISASMS_TILES = $(patsubst amethyst_tiles/shaders/%,amethyst_tiles/compiled/%.spvasm,$(SHADERS_TILES))


all: $(COMP_SHADERS) $(COMP_DISASMS) $(COMP_SHADERS_UI) $(COMP_DISASMS_UI)  $(COMP_SHADERS_TILES) $(COMP_DISASMS_TILES)

amethyst_rendy/compiled/%.spv: amethyst_rendy/shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -c -O -o $@ $<

amethyst_rendy/compiled/%.spvasm: amethyst_rendy/shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -S -g -O -o $@ $<

amethyst_tiles/compiled/%.spv: amethyst_tiles/shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -c -O -o $@ $<

amethyst_tiles/compiled/%.spvasm: amethyst_tiles/shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -S -g -O -o $@ $<

amethyst_ui/compiled/%.spv: amethyst_ui/shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -c -O -o $@ $<

amethyst_ui/compiled/%.spvasm: amethyst_ui/shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -S -g -O -o $@ $<

clean:
	rm amethyst_rendy/compiled/**/*.spv amethyst_rendy/compiled/**/*.spvasm amethyst_rendy/compiled/**/*.d
	rm amethyst_ui/compiled/*.spv amethyst_ui/compiled/*.spvasm amethyst_ui/compiled/*.d
	rm amethyst_tiles/compiled/*.spv amethyst_tiles/compiled/*.spvasm amethyst_tiles/compiled/*.d

.PHONY: all clean