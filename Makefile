# This Makefile is meant for shader compilation only.
# Use cargo to compile the rust part of the project.

GLSLC = $(shell ./find_glslc.sh)
ifeq "$(GLSLC)" ""
	break;
endif

define outpath
$(addsuffix .$(1), $(subst /shaders/,/compiled/,$(2)))
endef

SHADERS = $(filter-out /header/,$(wildcard */shaders/**/*.vert */shaders/**/*.frag))
OUT = $(call outpath,spv,$(SHADERS)) $(call outpath,spvasm,$(SHADERS))

all: $(OUT)

%.spv:
	mkdir -p $(dir $@)
	$(GLSLC) -MD -c -g -O -o $@ $<

%.spvasm:
	mkdir -p $(dir $@)
	$(GLSLC) -MD -S -g -O -o $@ $<

define shader_rules
$(call outpath,spv,$1): $1
$(call outpath,spvasm,$1) : $1
endef

$(foreach shader,$(SHADERS),$(eval $(call shader_rules,$(shader))))

clean:
	$(RM) */compiled/**/*.spv */compiled/**/*.spvdis */compiled/**/*.d

.PHONY: all clean