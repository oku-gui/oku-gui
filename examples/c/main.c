#include "oku_c.h"

#include <stdio.h>
#include <string.h>

ByteBox state() {
    char* a = "fff";
    return new_byte_box(a, strlen(a));
}

ComponentSpecification view(const void* state) {
    Style style = default_styles();
    style.color = color_rgba(255, 0, 0, 255);

    Container container = new_container(NULL, 0);

    Text text = new_text("Foo");
    text.style = style;

    return text_to_component(&text);
}

CUpdateResult update(const void* state, CEvent event) {

    if (event.message.tag == OkuMessage && event.message.oku_message.tag == PointerButtonEvent) {
        CPointerButton pointer_button_event = event.message.oku_message.pointer_button_event;

        if (pointer_button_event.state == Pressed) {
            printf("Button clicked at position (%.2f, %.2f) with state {%s} \n", pointer_button_event.position.x, pointer_button_event.position.y, state);
        }
    }

    return new_update_result();
}

int main() {
    ComponentData comp_data;

    comp_data.tag = "example_tag";
    comp_data.default_state = state;
    comp_data.default_props = state;
    comp_data.view_fn = view;
    comp_data.update_fn = update;

    ComponentSpecification spec = component(comp_data);

    OkuOptions options;
    options.renderer = 2;
    options.window_title = "Hello Oku C";

    oku_main(spec, &options);

    return 0;
}
