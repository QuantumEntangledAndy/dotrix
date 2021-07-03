//! Entity Component System

use std::hash::Hash;

use core::ops::{ Deref, DerefMut };

use crate::application::{ Services, IntoService };

/// Entity structure has only id field and represent an agregation of components
#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Entity(u64);

impl From<u64> for Entity {
    fn from(id: u64) -> Self {
        Entity(id)
    }
}

/// Any data structure can be a component
pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

/// System Priority
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Priority {
    /// Low priority (1024)
    Low,
    /// Normal priority (2048)
    Normal,
    /// High priority (4096)
    High,
    /// User defined value
    Custom(u32),
}

impl From<Priority> for u32 {
    fn from(priority: Priority) -> u32 {
        match priority {
            Priority::Low => 1024,
            Priority::Normal => 2048,
            Priority::High => 4096,
            Priority::Custom(value) => value,
        }
    }
}

/// Wrapper for system functions
pub struct System {
    /// Boxed system data
    pub data: Box<dyn Systemized>,
    /// System run level
    pub run_level: RunLevel,
}

/// Defines when and how often a system should run.
#[derive(Debug, Eq, PartialEq)]
pub enum RunLevel {
    /// Only once on application startup
    Startup,
    /// Begin of rendering request handling
    Bind,
    /// Before rendering stage
    Standard,
    /// Rendering stage
    Render,
    /// End of the rendering request handling
    Release,
    /// Execution after window resizing
    Resize,
}

impl From<&str> for RunLevel {
    fn from(name: &str) -> Self {
        if name.ends_with("::startup") {
            RunLevel::Startup
        } else if name.ends_with("::bind") {
            RunLevel::Bind
        } else if name.ends_with("::render") {
            RunLevel::Render
        } else if name.ends_with("::release") {
            RunLevel::Release
        } else if name.ends_with("::resize") {
            RunLevel::Resize
        } else {
            RunLevel::Standard
        }
    }
}

impl System {
    /// Constructs a system from function at default runlevel
    pub fn from<Fun, Ctx, Srv>(func: Fun) -> Self
    where
        Fun: IntoSystem<Ctx, Srv> + Sync + Send,
        Ctx: Sync + Send,
        Srv: Sync + Send,
    {
        let data = func.into_system();
        let run_level = RunLevel::from(data.name());

        Self {
            data,
            run_level,
        }
    }

    /// Adds an option to the system
    pub fn with<T>(mut self, option: T) -> Self
    where
        Self: SystemOption<T>,
    {
        self.set_option(option);
        self
    }
}

/// System option abstract interface
pub trait SystemOption<T> {
    /// customize the syystem with an option
    fn set_option(&mut self, option: T);
}

/// [`RunLevel`] option implementation
impl SystemOption<RunLevel> for System {
    fn set_option(&mut self, option: RunLevel) {
        self.run_level = option;
    }
}

/// [`RunLevel`] option implementation
impl SystemOption<Priority> for System {
    fn set_option(&mut self, option: Priority) {
        self.data.set_priority(option);
    }
}

struct SystemData<Run, Ctx> 
where
    Run: FnMut(&mut Ctx, &mut Services) + Send + Sync,
{
    name: &'static str,
    run: Run,
    ctx: Ctx,
    priority: Priority,
}

/// Abstraction for [`System`] prepared to be integrated into engine
pub trait Systemized: Send + Sync {
    /// Returns name of the system
    fn name(&self) -> &'static str;
    /// Executes system cylce
    fn run(&mut self, app: &mut Services);
    /// Returns priority of the system
    fn priority(&self) -> Priority;
    /// Sets priority for the system
    fn set_priority(&mut self, priority: Priority);
}

impl<Run, Ctx> Systemized for SystemData<Run, Ctx>
where
    Run: FnMut(&mut Ctx, &mut Services) + Send + Sync,
    Ctx: SystemContext,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&mut self, app: &mut Services) {
        (self.run)(&mut self.ctx, app);
    }

    fn priority(&self) -> Priority {
        self.priority
    }

    fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }
}

/// Abstraction for a function that can be turned into a [`System`]
pub trait IntoSystem<C, S> {
    /// Converts a function into [`System`]
    fn into_system(self) -> Box<dyn Systemized>;
}

macro_rules! impl_into_system {
    (($($context: ident),*), ($($i: ident),*)) => {

        impl<Fun, $($context,)* $($i,)*> IntoSystem<($($context,)*), ($($i,)*)> for Fun
        where
            Fun: FnMut($(Context<$context>,)* $($i,)*) + Send + Sync + 'static,
            $($context: SystemContext,)*
            $($i: Accessor,)*
        {
            #[allow(non_snake_case)]
            #[allow(unused)]
            fn into_system(mut self: Fun) -> Box<dyn Systemized>
            {
                let data: SystemData<_, ($($context)*)> = SystemData {
                    name: std::any::type_name::<Fun>(),
                    run: move |ctx, app| {
                        // context (none or one)
                        $(
                            let $context = Context::new(ctx);
                        )*
                        // services
                        $(
                            let $i = $i::fetch(app);
                        )*
                        (self)($($context,)* $($i,)*);
                    },
                    priority: Priority::Normal,
                    ctx: ($($context::default())*),
                };
                Box::new(data)
            }
        }
    }
}

/// Abstraction for [`System`] context
pub trait SystemContext: Default + Send + Sync + 'static {}
impl<T: Default + Send + Sync + 'static> SystemContext for T {}

impl_into_system!((), (A));
impl_into_system!((), (A, B));
impl_into_system!((), (A, B, C));
impl_into_system!((), (A, B, C, D));
impl_into_system!((), (A, B, C, D, E));
impl_into_system!((), (A, B, C, D, E, F));
impl_into_system!((), (A, B, C, D, E, F, G));
impl_into_system!((), (A, B, C, D, E, F, G, H));

impl_into_system!((CTX), (A));
impl_into_system!((CTX), (A, B));
impl_into_system!((CTX), (A, B, C));
impl_into_system!((CTX), (A, B, C, D));
impl_into_system!((CTX), (A, B, C, D, E));
impl_into_system!((CTX), (A, B, C, D, E, F));
impl_into_system!((CTX), (A, B, C, D, E, F, G));
impl_into_system!((CTX), (A, B, C, D, E, F, G, H));

/// Accessor for [`System`] context to privately store data between runs
pub struct Context<T> {
    value: *mut T,
}

impl<T> Context<T> {
    /// Constructs an accessor instance
    fn new(ctx: &mut T) -> Self {
        Context {
            value: ctx as *mut T,
        }
    }
}

impl<T> Deref for Context<T>
where
    T: SystemContext,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.value
        }
    }
}

impl<T> DerefMut for Context<T>
where
    T: SystemContext,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value
        }
    }
}

/// Mutable accessor for [`IntoService`] instance
pub struct Mut<T>
where
    T: IntoService
{
    value: *mut T,
}

impl<T> Deref for Mut<T>
where
    T: IntoService,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.value
        }
    }
}

impl<T> DerefMut for Mut<T>
where
    T: IntoService,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value
        }
    }
}

unsafe impl<T: IntoService> Send for Mut<T> {}
unsafe impl<T: IntoService> Sync for Mut<T> {}

/// Imutable accessor for a Service instance
pub struct Const<T> {
    value: *const T,
}

impl<T> Deref for Const<T>
where
    T: IntoService,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.value
        }
    }
}

unsafe impl<T: IntoService> Send for Const<T> {}
unsafe impl<T: IntoService> Sync for Const<T> {}

/// Abstraction to access Service in the storage
pub trait Accessor: Send + Sync {
    /// Type of Service to be accessed
    type Item: IntoService;
    /// Fetches the Service from the storage
    fn fetch(services: &mut Services) -> Self;
}

impl<T> Accessor for Mut<T>
where
    T: IntoService,
{
    type Item = T;
    fn fetch(services: &mut Services) -> Self {
        let service: &mut T = services.get_mut::<T>()
            .unwrap_or_else(|| panic!("Service {} does not exist", std::any::type_name::<T>()));
        Mut {
            value: service as *mut T
        }
    }
}

impl<T> Accessor for Const<T>
where
    T: IntoService,
{
    type Item = T;
    fn fetch(service: &mut Services) -> Self {
        let service: &T = service.get::<T>()
            .unwrap_or_else(|| panic!("Service {} does not exist", std::any::type_name::<T>()));
        Const {
            value: service as *const T
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::Services,
        ecs::{ Context, System, Const, Mut, RunLevel },
        world::World,
    };

    struct MyComponent(u64);

    fn my_system(mut world: Mut<World>) {
        world.spawn(Some((MyComponent(123),)));
    }

    #[test]
    fn world_system() {
        let mut services = Services::new();
        services.add(World::new());
        let mut s = System::from(my_system);
        s.data.run(&mut services);
        assert_eq!(services.get::<World>().unwrap().counter(), 1);
    }

    #[derive(Default)]
    struct MyContext(u64);

    struct MyService {
        data: u64
    }

    fn my_system_with_context(ctx: Context<MyContext>, mut service: Mut<MyService>) {
        service.data = ctx.0;
    }

    #[test]
    fn custom_system() {
        let mut services = Services::new();
        services.add(MyService { data: 123 });
        let mut s = System::from(my_system_with_context);
        s.data.run(&mut services);
        assert_eq!(services.get::<MyService>().unwrap().data, 0);
    }

    fn startup(_service: Const<MyService>) {}
    fn bind(_service: Const<MyService>) {}
    fn render(_service: Const<MyService>) {}
    fn release(_service: Const<MyService>) {}
    fn resize(_service: Const<MyService>) {}

    #[test]
    fn system_runlevel_autodetect() {
        let system = System::from(startup);
        assert_eq!(system.run_level, RunLevel::Startup);

        let system = System::from(bind);
        assert_eq!(system.run_level, RunLevel::Bind);

        let system = System::from(render);
        assert_eq!(system.run_level, RunLevel::Render);

        let system = System::from(release);
        assert_eq!(system.run_level, RunLevel::Release);

        let system = System::from(resize);
        assert_eq!(system.run_level, RunLevel::Resize);

        let system = System::from(my_system_with_context);
        assert_eq!(system.run_level, RunLevel::Standard);
    }
}
