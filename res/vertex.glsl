#version 330 core
layout(location = 0) in vec2 pos;

uniform vec2 window_size;
uniform vec4 location;
uniform vec4 uv_rect;

out vec2 uv;

void main() {
	vec2 positioned = vec2(pos.x, 1-pos.y) * location.xy + location.zw;
	positioned.y = window_size.y - positioned.y;
	vec2 screen = positioned / window_size * 2 - 1;
	
	gl_Position = vec4(screen, 0, 1);
	uv = vec2(pos.x, 1-pos.y) * uv_rect.xy + uv_rect.zw;
}
