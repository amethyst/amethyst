
GLSLC = $(shell ./find_glslc.sh)
ifeq "$(GLSLC)" ""
	break;
endif

FLAGS = -c -g

SHADERS=$(wildcard shaders/**/*)
COMP_SHADERS = $(patsubst shaders/%,compiled/%.spv,$(SHADERS))
COMP_DISASMS = $(patsubst shaders/%,compiled/%.spvasm,$(SHADERS))

all: $(COMP_SHADERS) $(COMP_DISASMS)

compiled/%.spv: shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -c -g -O -o $@ $<

compiled/%.spvasm: shaders/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -S -g -O -o $@ $<

clean:
	rm compiled/**/*.spv compiled/**/*.spvasm compiled/**/*.d

.PHONY: all clean