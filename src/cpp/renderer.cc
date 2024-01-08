#include "renderer.h"
#include <OpenGL/gl3.h>
#include <cstdint>
#include <iostream>
#include <vector>

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

void renderer_setup(GLuint* vao, GLuint* ebo, GLuint* vbo_instance, GLuint* tex_id) {
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC1_COLOR, GL_ONE_MINUS_SRC1_COLOR);
    glDepthMask(GL_FALSE);

    glGenVertexArrays(1, vao);
    glGenBuffers(1, ebo);
    glGenBuffers(1, vbo_instance);
    glBindVertexArray(*vao);

    GLuint indices[] = {0, 1, 3, 1, 2, 3};
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, *ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, 6 * 4, indices, GL_STATIC_DRAW);

    glBindBuffer(GL_ARRAY_BUFFER, *vbo_instance);
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
    glGenTextures(1, tex_id);
    glBindTexture(GL_TEXTURE_2D, *tex_id);

    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, 1024, 1024, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    glBindTexture(GL_TEXTURE_2D, 0);
}

void draw(GLuint vao, GLuint ebo, GLuint vbo_instance, GLuint tex_id, GLuint shader_program) {
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
    instances.push_back(InstanceData{0, 10, 24, 3, 15, 24, 0.0, 0.0, 0.0146484375, 0.0234375});

    glBufferSubData(GL_ARRAY_BUFFER, 0, 28, instances.data());
    glBindTexture(GL_TEXTURE_2D, tex_id);
    glDrawElementsInstanced(GL_TRIANGLES, 6, GL_UNSIGNED_INT, nullptr, 1);

    std::cout << glGetString(GL_VERSION) << '\n';
}
