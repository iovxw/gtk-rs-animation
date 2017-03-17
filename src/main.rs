extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate glib;
extern crate cairo;
extern crate gtk_animator;

use std::time::Duration;

use gtk::prelude::*;

const WINDOW_WIDTH: i32 = 400;
const WINDOW_HEIGHT: i32 = 400;
const BALL_RADIUS: i32 = 50;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Example");
    window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    window.set_app_paintable(true);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let layout = gtk::Layout::new(None, None);
    window.add(&layout);

    let ball = gtk::DrawingArea::new();
    ball.set_size_request(BALL_RADIUS * 2, BALL_RADIUS * 2);
    ball.connect_draw(|ball, cr| {
        let (width, _height) = ball.get_size_request();
        let radius = width as f64 / 2.0;

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.arc(radius, radius, radius, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill();

        Inhibit(false)
    });
    let ball_animator = gtk_animator::Animator::new(Duration::from_secs(5),
                                                    gtk_animator::Repeat::function(|| {
                                                        println!("repeat!");
                                                        true
                                                    }),
                                                    {
                                                        let layout = layout.clone();
                                                        let ball = ball.clone();
                                                        move |p, _p_raw| {
                                                            layout.set_child_y(&ball,
                                                                               (400.0 * p) as i32);
                                                        }
                                                    },
                                                    |p| p);

    ball.connect_show({
        let ball_animator = ball_animator.clone();
        move |_ball| { ball_animator.start(); }
    });
    layout.put(&ball,
               WINDOW_WIDTH / 2 - BALL_RADIUS,
               WINDOW_HEIGHT / 2 - BALL_RADIUS);

    let buttons = gtk::ButtonBox::new(gtk::Orientation::Vertical);
    layout.add(&buttons);

    let start_btn = gtk::Button::new_with_label("Start");
    start_btn.connect_clicked({
        let ball_animator = ball_animator.clone();
        move |_| { ball_animator.start(); }
    });
    buttons.add(&start_btn);

    let pause_btn = gtk::Button::new_with_label("Pause");
    pause_btn.connect_clicked({
        let ball_animator = ball_animator.clone();
        move |_| { ball_animator.pause(); }
    });
    buttons.add(&pause_btn);

    let reset_btn = gtk::Button::new_with_label("Reset");
    reset_btn.connect_clicked({
        let ball_animator = ball_animator.clone();
        move |_| { ball_animator.reset(); }
    });
    buttons.add(&reset_btn);

    let finish_btn = gtk::Button::new_with_label("Finish");
    finish_btn.connect_clicked({
        let ball_animator = ball_animator.clone();
        move |_| { ball_animator.finish(); }
    });
    buttons.add(&finish_btn);

    let reverse_btn = gtk::Button::new_with_label("Reverse");
    reverse_btn.connect_clicked({
        let ball_animator = ball_animator.clone();
        move |_| { ball_animator.reverse(!ball_animator.is_reversing()); }
    });
    buttons.add(&reverse_btn);

    window.show_all();

    gtk::main();
}
