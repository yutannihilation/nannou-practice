# All input shaders.
vert = $(wildcard src/shaders/*.vert)
frag = $(wildcard src/shaders/*.frag)

spirv = $(addsuffix .spv,$(vert) $(frag))

default: all

all: $(spirv)

%.vert.spv: %.vert
	glslangValidator -V $< -o $@

%.frag.spv: %.frag
	glslangValidator -V $< -o $@

clean:
	rm -f $(spirv)

.PHONY: default clean all