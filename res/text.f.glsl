#version 330 core

in vec2 TexCoords;
flat in vec4 fg;
flat in vec4 bg;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

uniform sampler2D mask;

void main() {
    float colored = fg.a;

    // The wide char information is already stripped, so it's safe to check for equality here.
    if (int(colored) == 1) {
        // Color glyphs, like emojis.
        color = texture(mask, TexCoords);
        alphaMask = vec4(color.a);

        // Revert alpha premultiplication.
        if (color.a != 0.0) {
            color.rgb = vec3(color.rgb / color.a);
        }

        color = vec4(color.rgb, 1.0);
    } else {
        // Regular text glyphs.
        vec3 textColor = texture(mask, TexCoords).rgb;
        alphaMask = vec4(textColor, textColor.r);
        color = vec4(fg.rgb, 1.0);
    }
}
