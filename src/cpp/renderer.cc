#include "renderer.h"
#include <OpenGL/gl3.h>

void renderer_setup(GLuint* vao, GLuint* ebo, GLuint* vbo_instance) {
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
}
