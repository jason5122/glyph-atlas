#define float_t float
#define color_t vec4

out vec4 FragColor;
#define FRAG_COLOR FragColor

flat in color_t color;

uniform float_t cellWidth;
uniform float_t cellHeight;
uniform float_t paddingY;
uniform float_t paddingX;

void main() {
    float_t x = floor(mod(gl_FragCoord.x - paddingX, cellWidth));
    float_t y = floor(mod(gl_FragCoord.y - paddingY, cellHeight));

    FRAG_COLOR = color;
}
