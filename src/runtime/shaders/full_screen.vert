#version 450 core

// This shader draws a giant triangle just so I can create a fragment shader over the entire canvas

void main() {
	const vec4 vertices[3] = vec4[3](vec4(0., 10., 0.5, 1.0),
									 vec4(-10., -1., 0.5, 1.0),
									 vec4( 10.,  -1., 0.5, 1.0));
                                     
	gl_Position = vertices[gl_VertexID];
}