use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{sleep, spawn};
use std::time::Duration;
use adw::gdk::Display;
use adw::gio;
use gtk::prelude::*;
use gtk::{glib, ApplicationWindow, Label, Box, Orientation, CssProvider, Button};
use gtk::glib::clone;

const APPLICATION_ID: &str = "org.strawberryfoundations.app.benchmark";
const VERSION: &str = "1.0.0";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APPLICATION_ID).build();

    app.connect_startup(|_| load_css());
    app.connect_activate(main_ui);

    app.run()
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_bytes(&glib::Bytes::from(include_bytes!("style.css")));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
fn main_ui(app: &adw::Application) {
    let running = Arc::new(AtomicBool::new(true));
    let running_mthread = running.clone();

    let bench_time = 10;

    let title = Label::builder()
        .label("Benchmark")
        .margin_top(10)
        .build();

    let subtitle = Label::builder()
        .label(format!("v{VERSION} - By Juliandev02 x Strawberry Foundations"))
        .margin_top(10)
        .build();

    let start_button_sthread = Button::builder()
        .label("Start Benchmark (Single-threaded)")
        .margin_top(35)
        .width_request(50)
        .margin_start(150)
        .margin_end(150)
        .build();

    let start_button_mthread = Button::builder()
        .label("Start Benchmark (Multi-threaded)")
        .margin_top(15)
        .width_request(50)
        .margin_start(150)
        .margin_end(150)
        .build();

    let counter_label = Label::builder()
        .label("")
        .margin_top(15)
        .width_request(50)
        .margin_start(150)
        .margin_end(150)
        .build();

    let result_label_title = Label::builder()
        .label("")
        .margin_top(15)
        .width_request(50)
        .margin_start(150)
        .margin_end(150)
        .build();

    let result_label = Label::builder()
        .label("")
        .margin_top(15)
        .width_request(50)
        .margin_start(150)
        .margin_end(150)
        .build();



    title.add_css_class("header");
    subtitle.add_css_class("subtitle");
    counter_label.add_css_class("default");
    result_label_title.add_css_class("default--large--bold");

    start_button_sthread.add_css_class("sthread");
    start_button_mthread.add_css_class("mthread");

    let cl_clone = counter_label.clone();
    let res_clone = result_label.clone();
    let res_title_clone = result_label_title.clone();

    let (sender, receiver) = async_channel::bounded(1);
    let (csend, crec) = async_channel::bounded(1);
    let (bsend, brec) = async_channel::bounded(1);

    let clone_start_button_mthread = start_button_mthread.clone();

    // Handler for clicking on start button
    start_button_sthread.connect_clicked(move |button| {
        button.set_sensitive(false);
        clone_start_button_mthread.set_sensitive(false);

        // Clone and define some important variables that are required by the runtime
        let running_rtm = running.clone();
        let running_loader = running.clone();
        let cl_clone_2 = cl_clone.clone();

        let sender = sender.clone();
        let receiver = receiver.clone();

        let csend = csend.clone();
        let crec = crec.clone();

        let bsend = bsend.clone();
        let brec = brec.clone();

        let mut x: u64 = 0;

        gio::spawn_blocking(move || {
            sender.send_blocking(false).expect("The channel needs to be open.");
            for i in (0..4).rev() {
                sleep(Duration::from_secs(1));
                csend.send_blocking(format!("Starting in {i}s")).expect("The channel needs to be open.");
            }
            sleep(Duration::from_secs(1));
            csend.send_blocking("Starting Benchmark...".to_string()).expect("The channel needs to be open.");
            sender.send_blocking(true).expect("The channel needs to be open.");
        });

        glib::spawn_future_local(clone!(@weak cl_clone_2 => async move {
            while let Ok(val) = crec.recv().await {
                cl_clone_2.set_label(val.as_str());
            }
        }));

        // Thread for sleeping x-seconds (benchmark_time) and after that setting the value of running to false
        glib::spawn_future_local(clone!(@weak cl_clone_2 => async move {
            while let Ok(b) = receiver.recv().await {
                let running_rtm = running_rtm.clone();
                let bsend = bsend.clone();

                if b {
                    spawn(move || {
                        bsend.send_blocking(true).expect("The channel needs to be open.");
                        sleep(Duration::from_secs(bench_time));
                        running_rtm.store(false, Ordering::Relaxed);
                    });
                }
            }
        }));


        // Runtime
        glib::spawn_future_local(clone!(@weak button, @weak cl_clone, @weak res_clone, @weak res_title_clone, @weak cl_clone_2 => async move {
            while let Ok(b) = brec.recv().await {
                if b {
                    let running_loader = running_loader.clone();

                    let runtime = gio::spawn_blocking(move || {
                        while running_loader.load(Ordering::Relaxed) {
                            x += 1;
                        }
                        (true, x)
                    }).await.expect("Task needs to finish successfully.");

                    let (button_lock, x) = runtime;

                    let bench_time_ms = bench_time * 1000;
                    let result = (x + bench_time_ms) / 900_000;

                    button.set_sensitive(button_lock);
                    cl_clone.set_label("Benchmark finished");
                    res_title_clone.set_label("Your results are here!");
                    res_clone.set_label(format!("Your computer has scored {} points!", result).as_str());
                }
            }
        }));
    });

    let cl_clone = counter_label.clone();
    let res_clone = result_label.clone();
    let res_title_clone = result_label_title.clone();

    let (sender, receiver) = async_channel::bounded(1);
    let (csend, crec) = async_channel::bounded(1);
    let (bsend, brec) = async_channel::bounded(1);

    let clone_start_button_sthread = start_button_sthread.clone();

    // Handler for clicking on start button
    start_button_mthread.connect_clicked(move |button| {
        button.set_sensitive(false);
        clone_start_button_sthread.set_sensitive(false);

        // Clone and define some important variables that are required by the runtime
        let running_rtm = running_mthread.clone();
        let running_loader = running_mthread.clone();
        let cl_clone_2 = cl_clone.clone();

        let sender = sender.clone();
        let receiver = receiver.clone();

        let csend = csend.clone();
        let crec = crec.clone();

        let bsend = bsend.clone();
        let brec = brec.clone();

        let num_threads = u8::try_from(std::thread::available_parallelism().unwrap().get()).unwrap();

        gio::spawn_blocking(move || {
            sender.send_blocking(false).expect("The channel needs to be open.");
            for i in (0..4).rev() {
                sleep(Duration::from_secs(1));
                csend.send_blocking(format!("Starting in {i}s")).expect("The channel needs to be open.");
            }
            sleep(Duration::from_secs(1));
            csend.send_blocking("Starting Benchmark...".to_string()).expect("The channel needs to be open.");
            sender.send_blocking(true).expect("The channel needs to be open.");
        });

        glib::spawn_future_local(clone!(@weak cl_clone_2 => async move {
            while let Ok(val) = crec.recv().await {
                cl_clone_2.set_label(val.as_str());
            }
        }));

        // Thread for sleeping x-seconds (benchmark_time) and after that setting the value of running to false
        glib::spawn_future_local(clone!(@weak cl_clone_2 => async move {
            while let Ok(b) = receiver.recv().await {
                let running_rtm = running_rtm.clone();
                let bsend = bsend.clone();

                if b {
                    spawn(move || {
                        bsend.send_blocking(true).expect("The channel needs to be open.");
                        sleep(Duration::from_secs(bench_time));
                        running_rtm.store(false, Ordering::Relaxed);
                    });
                }
            }
        }));


        // Runtime
        glib::spawn_future_local(clone!(@weak button, @weak cl_clone, @weak res_clone, @weak res_title_clone, @weak cl_clone_2 => async move {
            let running_loader = running_loader.clone();

            while let Ok(b) = brec.recv().await {
                if b {
                    let mut handles = vec![];
                    let running_loader = running_loader.clone();
                    let mut results = Vec::new();

                    for _ in 0..num_threads {
                        let running_clone = running_loader.clone();
                        handles.push(spawn(move || {
                            let mut c: u64 = 0;

                            while running_clone.load(Ordering::Relaxed) {
                                c += 1;
                            }

                            c
                        }));
                    }

                    for handle in handles {
                        results.push(handle.join().unwrap());
                    }


                    let total_count: u64 = results.iter().sum();
                    let bench_time_ms = bench_time * 1000;
                    let result = (total_count + bench_time_ms) / 900_000;

                    button.set_sensitive(true);
                    cl_clone.set_label("Benchmark finished");
                    res_title_clone.set_label("Your results are here!");
                    res_clone.set_label(format!("Your computer has scored {} points!", result).as_str());
                }
            }
        }));
    });

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    main_box.append(&title);
    main_box.append(&subtitle);
    main_box.append(&start_button_sthread);
    main_box.append(&start_button_mthread);
    main_box.append(&counter_label);
    main_box.append(&result_label_title);
    main_box.append(&result_label);

    let main_window = ApplicationWindow::builder()
        .application(app)
        .title("Benchmark")
        .width_request(500)
        .height_request(500)
        .default_width(500)
        .default_height(500)
        .child(&main_box)
        .build();

    main_window.present();
}