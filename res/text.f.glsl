#version 330 core

in vec2 TexCoords;
flat in vec4 fg;
flat in vec4 bg;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

#define COLORED 1

uniform int renderingPass;
uniform sampler2D mask;

void main() {
	if (renderingPass == 0) {
		if (bg.a == 0.0) {
			discard;
		}

		alphaMask = vec4(1.0);
		color = vec4(bg.rgb * bg.a, bg.a);
		return;
	}

	float colored = fg.a;

	if (int(colored) == COLORED) {
		color = texture(mask, TexCoords);
		alphaMask = vec4(color.a);

		if (color.a != 0.0) {
			color.rgb = vec3(color.rgb / color.a);
		}

		color = vec4(color.rgb, 1.0);
	} else {
		vec3 textColor = texture(mask, TexCoords).rgb;
		alphaMask = vec4(textColor, textColor.r);
		color = vec4(fg.rgb, 1.0);
	}
}
