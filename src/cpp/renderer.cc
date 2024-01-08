#include "renderer.h"
#include <OpenGL/gl3.h>

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

void draw(GLuint vao, GLuint ebo, GLuint vbo_instance, GLuint tex_id, GLuint shader_program,
          GLint u_projection, GLint u_cell_dim) {
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
}
