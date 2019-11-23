use nannou::prelude::*;

fn main() {
    nannou::app(model).view(view).run();
}

struct Model;

fn model(app: &App) -> Model {
    let i = 50;
    let f = 36.6;
    let b = true;
    let c = '!';
    let message = "hello world";

    println!("i = {}", i);
    println!("f = {}", f);
    println!("b = {}", b);
    println!("c = {}", c);
    println!("message = {}", message);

    app.new_window().with_dimensions(640, 480).build().unwrap();

    Model
}

fn view(_app: &App, _model: &Model, frame: &Frame) {
    frame.clear(DIMGRAY);
}
