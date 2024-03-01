use crate::*;

use std::any::TypeId;

// pub struct Events {
//     pub events: HashMap<TypeId, VecDeque<Box<dyn Any>>>,
// }
//
// impl Events {
//     pub fn new() -> Self {
//         Self { events: HashMap::default() }
//     }
//
//     pub fn send<T: 'static>(&mut self, event: T) {
//         self.events
//             .entry(TypeId::of::<T>())
//             .or_insert_with(VecDeque::default)
//             .push_back(Box::new(event));
//     }
//
//     pub fn get<T: 'static>(&mut self) -> &mut VecDeque<Box<&mut T>> {
//         let queue = self
//             .events
//             .entry(TypeId::of::<T>())
//             .or_insert_with(VecDeque::default);
//
//         // ??? magic downcast
//     }
// }

pub struct Events {
    pub events: HashMap<TypeId, Box<dyn Any>>,
}

impl Events {
    pub fn new() -> Self {
        Self { events: HashMap::default() }
    }

    pub fn send<T: 'static>(&mut self, event: T) {
        let queue = self
            .events
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<VecDeque<T>>::default());

        if let Some(queue) = queue.downcast_mut::<VecDeque<T>>() {
            queue.push_back(event);
        }
    }

    pub fn get<T: 'static>(&mut self) -> &mut VecDeque<T> {
        self.events
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<VecDeque<T>>::default())
            .downcast_mut()
            .unwrap()
    }
}

mod tests {
    #[test]
    pub fn basic_usage() {
        use crate::Events;

        let mut events = Events::new();

        events.send(3);
        events.send(4);
        events.send(5);

        events.send("foo");
        events.send("bar");

        let things = events.get::<i32>().iter_mut().collect::<Vec<_>>();
        assert!(things.into_iter().eq(&[3, 4, 5]));

        let strings = events.get::<&str>().iter_mut().collect::<Vec<_>>();
        assert!(strings.into_iter().eq(&["foo", "bar"]));
    }
}

// TODO: look at resources and then delete this
// #[derive(Default)]
// pub struct Context {
//     resources: HashMap<std::any::TypeId, RefCell<Box<dyn std::any::Any>>>,
// }
//
// impl Context {
//     pub fn register_resource<T: 'static>(&mut self, res: T) {
//         self.resources
//             .insert(std::any::TypeId::of::<T>(), RefCell::new(Box::new(res)));
//     }
//
//     pub fn get_resource<T: 'static>(&self) -> impl Deref<Target = T> + '_ {
//         Ref::map(
//             self.resources
//                 .get(&std::any::TypeId::of::<T>())
//                 .as_ref()
//                 .expect("Resource not found")
//                 .borrow(),
//             |x| x.downcast_ref::<T>().unwrap(),
//         )
//     }
//
//     pub fn get_resource_mut<T: 'static>(&self) -> impl DerefMut<Target = T> + '_ {
//         RefMut::map(
//             self.resources
//                 .get(&std::any::TypeId::of::<T>())
//                 .as_ref()
//                 .expect("Resource not found")
//                 .borrow_mut(),
//             |x| x.downcast_mut::<T>().unwrap(),
//         )
//     }
// }
//
// struct OpenGlStuff {
//     data: i32,
// }
// struct PathfindingGrid {
//     more_data: Vec<Vec<u32>>,
// }
//
// pub fn test() {
//     let mut ctx = Context::default();
//     ctx.register_resource(OpenGlStuff { data: 42 });
//     ctx.register_resource(PathfindingGrid {
//         more_data: vec![vec![1, 2, 3]],
//     });
// }
//
// pub fn system(ctx: &Context) {
//     let opengl = ctx.get_resource::<OpenGlStuff>();
//     let mut pathfinding = ctx.get_resource_mut::<PathfindingGrid>();
//     println!("OpenGL stuff: {}", opengl.data);
//     pathfinding.more_data.push(vec![1, 2, 3, 4]);
// }
