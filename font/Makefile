# All input shaders.
vert = $(wildcard src/shaders/*.vert)
frag = $(wildcard src/shaders/*.frag)

spirv = $(addsuffix .spv,$(vert) $(frag))

default: all

all: $(spirv) cargo out.mp4 out.gif

cargo:
	cargo run --release

%.vert.spv: %.vert
	glslangValidator -V $< -o $@

%.frag.spv: %.frag
	glslangValidator -V $< -o $@

img/*.png: cargo

out.mp4: img/*.png
	ffmpeg -y -r 60 -i img/%03d.png -vcodec libx264 -pix_fmt yuv420p -an -r 30 -b:v 2000k $@

out.gif: out.mp4
	# c.f. https://nico-lab.net/optimized_256_colors_with_ffmpeg/, https://trac.ffmpeg.org/wiki/How%20to%20speed%20up%20/%20slow%20down%20a%20video
	ffmpeg -y -r 30 -t 10 -i $< -filter_complex 'fps=10,setpts=0.3*PTS,scale=w=iw/2:-1,split[a][b];[a]palettegen[pal];[b][pal]paletteuse' $@

clean:
	rm -f out.mp4 out.gif

.PHONY: default clean all