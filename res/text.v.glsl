#version 330 core

layout(location = 0) in vec2 gridCoords;
layout(location = 1) in vec4 glyph;
layout(location = 2) in vec4 uv;

out vec2 TexCoords;

uniform vec2 resolution;
uniform vec2 cellDim;

vec2 pixelToClipSpace(vec2 point) {
    point /= resolution;         // Normalize to [0.0, 1.0].
    point.y = 1.0 - point.y;     // Set origin to top left instead of bottom left.
    return (point * 2.0) - 1.0;  // Convert to [-1.0, 1.0].
}

void main() {
    vec2 glyphOffset = glyph.xy;
    vec2 glyphSize = glyph.zw;
    vec2 uvOffset = uv.xy;
    vec2 uvSize = uv.zw;

    vec2 cornerMask;
    cornerMask.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.;
    cornerMask.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.;

    // Position of cell from top-left.
    vec2 cellPosition = cellDim * gridCoords;
    glyphOffset.y = cellDim.y - glyphOffset.y;

    cellPosition += glyphOffset + cornerMask * glyphSize;
    cellPosition.x += 200;

    gl_Position = vec4(pixelToClipSpace(cellPosition), 0.0, 1.0);

    TexCoords = uvOffset + cornerMask * uvSize;
}
