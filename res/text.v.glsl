#version 330 core 

layout(location = 0) in vec2 gridCoords;
layout(location = 1) in vec4 glyph;
layout(location = 2) in vec4 uv;
layout(location = 3) in vec4 fgColor;
layout(location = 4) in vec4 bgColor;

out vec2 TexCoords;
flat out vec4 fg;
flat out vec4 bg;

uniform vec2 cellDim;
uniform vec4 projection;

uniform int renderPass;

#define WIDE_CHAR 2

void main() {
	vec2 projectionOffset = projection.xy;
	vec2 projectionScale = projection.zw;

	vec2 position;
	position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.;
	position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.;

	vec2 cellPosition = cellDim * gridCoords;

	fg = vec4(fgColor.rgb / 255.0, fgColor.a);
	bg = bgColor / 255.0;

	float occupiedCells = 1;
	if ((int(fg.a) >= WIDE_CHAR)) {
		occupiedCells = 2;
		fg.a = round(fg.a - WIDE_CHAR);
	}

	if (renderPass == 0) {
		vec2 backgroundDim = cellDim;
		backgroundDim.x *= occupiedCells;

		vec2 finalPosition = cellPosition + backgroundDim * position;
		gl_Position = 
			vec4(projectionOffset + projectionScale * finalPosition, 0.0, 1.0);

		TexCoords = vec2(0, 0);
	} else {
		vec2 glyphSize = glyph.zw;
		vec2 glyphOffset = glyph.xy;
		glyphOffset.y = cellDim.y - glyphOffset.y;

		vec2 finalPosition = cellPosition + glyphSize * position + glyphOffset;
		gl_Position = 
			vec4(projectionOffset + projectionScale * finalPosition, 0.0, 1.0);
		
		vec2 uvOffset = uv.xy;
		vec2 uvSize = uv.zw;
		TexCoords = uvOffset + position * uvSize;
	}
}
