#version 330 core

uniform sampler2D atlas;
in vec2 uv;

layout(location = 0, index = 0) out vec4 outColor0;
layout(location = 0, index = 1) out vec4 outColor1;

void main() {
	outColor0 = vec4(0,0,0,1);
	outColor1 = texture(atlas, uv);
}
