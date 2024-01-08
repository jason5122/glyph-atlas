#include "renderer.h"
#include <OpenGL/gl3.h>
#include <cstdint>
#include <iostream>
#include <vector>

GLuint setup_shaders();

struct InstanceData {
    uint16_t col;
    uint16_t row;

    int16_t left;
    int16_t top;
    int16_t width;
    int16_t height;

    float uv_left;
    float uv_bot;
    float uv_width;
    float uv_height;
};

void draw() {
    GLuint vao = 0;
    GLuint ebo = 0;
    GLuint vbo_instance = 0;
    GLuint tex_id = 0;

    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC1_COLOR, GL_ONE_MINUS_SRC1_COLOR);
    glDepthMask(GL_FALSE);

    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &ebo);
    glGenBuffers(1, &vbo_instance);
    glBindVertexArray(vao);

    GLuint indices[] = {0, 1, 3, 1, 2, 3};
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, 6 * 4, indices, GL_STATIC_DRAW);

    glBindBuffer(GL_ARRAY_BUFFER, vbo_instance);
    glBufferData(GL_ARRAY_BUFFER, 4096 * 28, nullptr, GL_STREAM_DRAW);

    glVertexAttribPointer(0, 2, GL_UNSIGNED_SHORT, GL_FALSE, 28, (void*)0);
    glEnableVertexAttribArray(0);
    glVertexAttribDivisor(0, 1);

    glVertexAttribPointer(1, 4, GL_SHORT, GL_FALSE, 28, (void*)4);
    glEnableVertexAttribArray(1);
    glVertexAttribDivisor(1, 1);

    glVertexAttribPointer(2, 4, GL_FLOAT, GL_FALSE, 28, (void*)12);
    glEnableVertexAttribArray(2);
    glVertexAttribDivisor(2, 1);

    glBindVertexArray(0);
    glBindBuffer(GL_ARRAY_BUFFER, 0);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);

    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
    glGenTextures(1, &tex_id);
    glBindTexture(GL_TEXTURE_2D, tex_id);

    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, 1024, 1024, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    glBindTexture(GL_TEXTURE_2D, 0);

    GLuint shader_program = setup_shaders();

    GLint u_projection = glGetUniformLocation(shader_program, "projection");
    GLint u_cell_dim = glGetUniformLocation(shader_program, "cellDim");

    glViewport(10, 10, 3436, 2082);
    glUseProgram(shader_program);
    glUniform4f(u_projection, -1.0, 1.0, 0.0005820722, -0.00096061477);
    glUniform2f(u_cell_dim, 20.0, 40.0);
    glUseProgram(0);

    glUseProgram(shader_program);
    glBindVertexArray(vao);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
    glBindBuffer(GL_ARRAY_BUFFER, vbo_instance);
    glActiveTexture(GL_TEXTURE0);

    std::vector<uint8_t> buffer = {
        77,  77,  77,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,
        84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,  84,
        84,  84,  84,  77,  77,  77,  0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 235, 235, 235, 0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 235, 235, 235, 0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        124, 124, 124, 83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,
        83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  83,  77,  77,  77,  0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        59,  59,  59,  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        59,  59,  59,  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        136, 136, 136, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100,
        100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 45,  45,  45,  0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 115, 115, 115, 0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 115, 115, 115, 0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 112, 112, 112, 67,  67,  67,  67,  67,  67,
        67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,  67,
        67,  67,  67,  30,  30,  30,  0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        59,  59,  59,  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        59,  59,  59,  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        59,  59,  59,  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        59,  59,  59,  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 59,  59,  59,  0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
        0,   0,   0,   0,   0,   0,   0,   0,   0,   235, 235, 235, 255, 255, 255, 255, 255, 255,
        188, 188, 188, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168,
        168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 168, 36,  36,  36,
        235, 235, 235, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 55,  55,  55,  235, 235, 235, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 55,  55,  55};

    glBindTexture(GL_TEXTURE_2D, tex_id);
    glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, 15, 24, GL_RGB, GL_UNSIGNED_BYTE, buffer.data());
    glBindTexture(GL_TEXTURE_2D, 0);

    std::vector<InstanceData> instances;
    instances.push_back(InstanceData{20, 20, 24, 3, 15, 24, 0.0, 0.0, 0.0146484375, 0.0234375});

    glBufferSubData(GL_ARRAY_BUFFER, 0, 28, instances.data());
    glBindTexture(GL_TEXTURE_2D, tex_id);
    glDrawElementsInstanced(GL_TRIANGLES, 6, GL_UNSIGNED_INT, nullptr, 1);

    std::cout << glGetString(GL_VERSION) << '\n';
}

GLuint setup_shaders() {
    GLuint vertexShader = glCreateShader(GL_VERTEX_SHADER);
    GLuint fragmentShader = glCreateShader(GL_FRAGMENT_SHADER);
    const GLchar* vertSource = R"(
    #version 330 core

// Cell properties.
layout(location = 0) in vec2 gridCoords;

// Glyph properties.
layout(location = 1) in vec4 glyph;

// uv mapping.
layout(location = 2) in vec4 uv;

out vec2 TexCoords;

// Terminal properties
uniform vec2 cellDim;
uniform vec4 projection;

void main() {
    vec2 glyphOffset = glyph.xy;
    vec2 glyphSize = glyph.zw;
    vec2 uvOffset = uv.xy;
    vec2 uvSize = uv.zw;
    vec2 projectionOffset = projection.xy;
    vec2 projectionScale = projection.zw;

    // Compute vertex corner position
    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.;

    // Position of cell from top-left
    vec2 cellPosition = cellDim * gridCoords;

    glyphOffset.y = cellDim.y - glyphOffset.y;

    vec2 finalPosition = cellPosition + glyphSize * position + glyphOffset;
    gl_Position = vec4(projectionOffset + projectionScale * finalPosition, 0.0, 1.0);

    TexCoords = uvOffset + position * uvSize;
}
)";
    const GLchar* fragSource = R"(
    #version 330 core

in vec2 TexCoords;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

uniform sampler2D mask;

void main() {
    vec3 textColor = texture(mask, TexCoords).rgb;
    alphaMask = vec4(textColor, textColor.r);
    color = vec4(51 / 255.0, 51 / 255.0, 51 / 255.0, 1.0);
}
)";
    glShaderSource(vertexShader, 1, &vertSource, nullptr);
    glShaderSource(fragmentShader, 1, &fragSource, nullptr);
    glCompileShader(vertexShader);
    glCompileShader(fragmentShader);

    GLuint shader_program = glCreateProgram();
    glAttachShader(shader_program, vertexShader);
    glAttachShader(shader_program, fragmentShader);
    glLinkProgram(shader_program);

    glDeleteShader(vertexShader);
    glDeleteShader(fragmentShader);
    return shader_program;
}
