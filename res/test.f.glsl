#version 330 core
out vec4 FragColor;
  
in vec2 TexCoord;
/* in vec3 ourColor; */

uniform sampler2D ourTexture;

void main()
{
    vec4 c;

    c = texture(ourTexture, TexCoord);
    FragColor = vec4(1.0, 1.0, 1.0, c.r);
    /* FragColor = vec4(ourColor, 1.0); */
}
