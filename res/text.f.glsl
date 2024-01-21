#version 330 core

in vec2 TexCoords;
flat in vec4 fg;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

uniform sampler2D mask;

void main() {
    vec4 texel = texture(mask, TexCoords);
    vec3 textColor = texel.rgb;

    float colored = fg.a;
    if (int(colored) == 1) {
        alphaMask = vec4(texel.a);

        // Revert alpha premultiplication.
        if (texel.a != 0.0) {
            textColor = textColor / texel.a;
        }

        color = vec4(textColor, 1.0);
    } else {
        vec3 black = vec3(51, 51, 51) / 255.0;
        vec3 yellow = vec3(249, 174, 88) / 255.0;
        vec3 blue = vec3(102, 153, 204) / 255.0;

        alphaMask = vec4(textColor, textColor.r);
        color = vec4(yellow, 1.0);
    }
}
