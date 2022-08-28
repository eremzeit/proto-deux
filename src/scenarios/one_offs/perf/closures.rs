use std::rc::Rc;

pub fn test_closure_perf() {
    let iterations = 400000;
    _test_closures(iterations);
    _test_functions(iterations);

    perf_timers_print!();
}

pub type ClosureFunction = dyn Fn(&mut SomeObject) -> bool;
pub struct SomeObject {
    pub count: usize,
}

// 1000 ticks * 20 units * (10 conditionals + 1 reaction call)

pub struct ClosureWrapper {
    pub function: Rc<ClosureFunction>,
}

fn func1(obj: &mut SomeObject) {
    obj.count += 1;
}
fn func2(obj: &mut SomeObject) {
    obj.count += 1;
}
fn func3(obj: &mut SomeObject) {
    obj.count += 1;
}
fn func4(obj: &mut SomeObject) {
    obj.count += 1;
}
fn func5(obj: &mut SomeObject) {
    obj.count += 1;
}

pub fn _test_functions(iterations: usize) {
    perf_timer_start!("functions");

    let mut obj = SomeObject { count: 0 };
    while obj.count < iterations {
        func1(&mut obj);
        func2(&mut obj);
        func3(&mut obj);
        func4(&mut obj);
        func5(&mut obj);
    }

    println!("{}", obj.count);

    perf_timer_stop!("functions");
}

pub fn _test_closures(iterations: usize) {
    let closures: Vec<Rc<ClosureFunction>> = vec![
        Rc::new(|some_obj: &mut SomeObject| -> bool {
            some_obj.count += 1;
            true
        }),
        Rc::new(|some_obj: &mut SomeObject| -> bool {
            some_obj.count += 1;
            true
        }),
        Rc::new(|some_obj: &mut SomeObject| -> bool {
            some_obj.count += 1;
            true
        }),
        Rc::new(|some_obj: &mut SomeObject| -> bool {
            some_obj.count += 1;
            true
        }),
        Rc::new(|some_obj: &mut SomeObject| -> bool {
            some_obj.count += 1;
            true
        }),
    ];

    perf_timer_start!("closures");

    let len = closures.len();

    let mut obj = SomeObject { count: 0 };
    while obj.count < iterations {
        for i in 0..len {
            closures[i](&mut obj);
        }
    }
    println!("{}", obj.count);

    perf_timer_stop!("closures");

    // let closures: Vec<ClosureWrapper> = vec![
    //     ClosureWrapper {
    //         function: Rc::new(|some_obj: &mut SomeObject| -> bool { true }),
    //     },
    //     ClosureWrapper {
    //         function: Rc::new(|some_obj: &mut SomeObject| -> bool { true }),
    //     },
    // ];
}
