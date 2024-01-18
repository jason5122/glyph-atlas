#version 330 core

// Cell properties.
layout(location = 0) in vec2 gridCoords;

out vec2 TexCoords;

uniform vec2 cellDim;

vec2 pixelToClipSpace(vec2 point) {
    point /= vec2(1728 * 2, 1051 * 2);  // Normalize to [0.0, 1.0].
    point.y = 1.0 - point.y;            // Set origin to top left instead of bottom left.
    return (point * 2.0) - 1.0;         // Convert to [-1.0, 1.0].
}

void main() {
    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 30. : 0.;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 30.;

    // Position of cell from top-left
    vec2 cellPosition = cellDim * gridCoords * 2;
    vec2 finalPosition = cellPosition + position;

    gl_Position = vec4(pixelToClipSpace(finalPosition), 0.0, 1.0);

    TexCoords = position;
}
