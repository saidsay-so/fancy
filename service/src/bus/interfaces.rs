// This code was autogenerated with `dbus-codegen-rust `, see https://github.com/diwic/dbus-rs
use dbus as dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus_tree as tree;

pub trait ComMusikidFancy {
    fn set_target_fan_speed(&self, index: u8, speed: f64) -> Result<(), tree::MethodErr>;
    fn fans_speeds(&self) -> Result<::std::collections::HashMap<String, f64>, tree::MethodErr>;
    fn target_fans_speeds(&self) -> Result<Vec<f64>, tree::MethodErr>;
    fn set_target_fans_speeds(&self, value: Vec<f64>) -> Result<(), tree::MethodErr>;
    fn config(&self) -> Result<String, tree::MethodErr>;
    fn set_config(&self, value: String) -> Result<(), tree::MethodErr>;
    fn auto(&self) -> Result<bool, tree::MethodErr>;
    fn set_auto(&self, value: bool) -> Result<(), tree::MethodErr>;
    fn critical(&self) -> Result<bool, tree::MethodErr>;
    fn temperatures(&self) -> Result<::std::collections::HashMap<String, f64>, tree::MethodErr>;
}

pub fn com_musikid_fancy_server<F, T, D>(factory: &tree::Factory<tree::MTFn<D>, D>, data: D::Interface, f: F) -> tree::Interface<tree::MTFn<D>, D>
where
    D: tree::DataType,
    D::Method: Default,
    D::Property: Default,
    T: ComMusikidFancy,
    F: 'static + for <'z> Fn(& 'z tree::MethodInfo<tree::MTFn<D>, D>) -> & 'z T,
{
    let i = factory.interface("com.musikid.fancy", data);
    let f = ::std::sync::Arc::new(f);
    let fclone = f.clone();
    let h = move |minfo: &tree::MethodInfo<tree::MTFn<D>, D>| {
        let mut i = minfo.msg.iter_init();
        let index: u8 = i.read()?;
        let speed: f64 = i.read()?;
        let d = fclone(minfo);
        d.set_target_fan_speed(index, speed)?;
        let rm = minfo.msg.method_return();
        Ok(vec!(rm))
    };
    let m = factory.method("SetTargetFanSpeed", Default::default(), h);
    let m = m.in_arg(("index", "y"));
    let m = m.in_arg(("speed", "d"));
    let i = i.add_m(m);

    let p = factory.property::<::std::collections::HashMap<&str, f64>, _>("FansSpeeds", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.fans_speeds()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<Vec<f64>, _>("TargetFansSpeeds", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.target_fans_speeds()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_target_fans_speeds(iter.read()?)?;
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<&str, _>("Config", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.config()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_config(iter.read()?)?;
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("Auto", Default::default());
    let p = p.access(tree::Access::ReadWrite);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.auto()?);
        Ok(())
    });
    let fclone = f.clone();
    let p = p.on_set(move |iter, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        d.set_auto(iter.read()?)?;
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<bool, _>("Critical", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.critical()?);
        Ok(())
    });
    let i = i.add_p(p);

    let p = factory.property::<::std::collections::HashMap<&str, f64>, _>("Temperatures", Default::default());
    let p = p.access(tree::Access::Read);
    let fclone = f.clone();
    let p = p.on_get(move |a, pinfo| {
        let minfo = pinfo.to_method_info();
        let d = fclone(&minfo);
        a.append(d.temperatures()?);
        Ok(())
    });
    let i = i.add_p(p);
    i
}