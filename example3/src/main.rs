use nannou::prelude::*;

const GLOBAL: i32 = 10;

fn main() {
    nannou::app(model).run();
}

struct Model {
    foo: i32,
    bar: f64,
}

fn model(app: &App) -> Model {
    app.new_window().event(event).view(view).build().unwrap();

    let foo = 80;
    let bar = 3.14;
    Model { foo, bar }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(_key) => {
            println!("foo = {}", model.foo);
            println!("bar = {}", model.bar);
        }
        KeyReleased(_key) => {
            let local_var = 94;
            println!("local_variable to KeyReleased = {}", local_var);
        }
        MousePressed(_button) => {
            println!("global scope: GLOBAL = {}", GLOBAL);
        }
        _other => (),
    }
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(RED);
}
