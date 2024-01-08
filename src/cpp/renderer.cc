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
}
