//! System tray integration.

#[cfg(feature = "appindicator-tray")]
use std::cell::RefCell;
#[cfg(not(feature = "appindicator-tray"))]
use std::cell::RefCell;
#[cfg(not(feature = "appindicator-tray"))]
use std::collections::BTreeMap;
#[cfg(feature = "appindicator-tray")]
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(feature = "appindicator-tray")]
use std::rc::Rc;
#[cfg(not(feature = "appindicator-tray"))]
use std::rc::Rc;

#[cfg(feature = "appindicator-tray")]
use anyhow::anyhow;
#[cfg(feature = "appindicator-tray")]
use anyhow::Context;
use anyhow::Result;
#[cfg(not(feature = "appindicator-tray"))]
use anyhow::{anyhow, Context};
#[cfg(not(feature = "appindicator-tray"))]
use gtk::glib::{self, variant::ToVariant};
#[cfg(feature = "appindicator-tray")]
use gtk3::prelude::*;
#[cfg(feature = "appindicator-tray")]
use libappindicator::{AppIndicator, AppIndicatorStatus};
#[cfg(feature = "appindicator-tray")]
use tokio::sync::broadcast;
#[cfg(feature = "appindicator-tray")]
use uuid::Uuid;

use crate::accounts::Account;
use crate::ipc::AppEvent;
use crate::ipc::EventBus;

#[cfg(feature = "appindicator-tray")]
pub(crate) mod icons;
#[cfg(feature = "appindicator-tray")]
pub(crate) mod menu;

/// Running tray indicator.
#[cfg(feature = "appindicator-tray")]
pub(crate) struct Tray {
    indicator: Rc<RefCell<AppIndicator>>,
    menu: gtk3::Menu,
}

#[cfg(not(feature = "appindicator-tray"))]
pub(crate) struct Tray {
    connection: gio::DBusConnection,
    registration_ids: Vec<gio::RegistrationId>,
    owner_id: Option<gio::OwnerId>,
}

#[cfg(feature = "appindicator-tray")]
impl Tray {
    /// Keeps the tray objects captured by GTK signal closures.
    pub(crate) fn keep_alive(&self) {
        let _ = self.indicator.borrow();
        let _ = self.menu.is_visible();
    }
}

#[cfg(not(feature = "appindicator-tray"))]
impl Tray {
    /// Keeps the D-Bus tray registration alive.
    pub(crate) fn keep_alive(&self) {
        let _ = self.connection.is_closed();
        let _ = self.registration_ids.len();
        let _ = self.owner_id.is_some();
    }
}

#[cfg(not(feature = "appindicator-tray"))]
impl Drop for Tray {
    fn drop(&mut self) {
        for registration_id in self.registration_ids.drain(..) {
            if let Err(error) = self.connection.unregister_object(registration_id) {
                tracing::debug!(?error, "failed to unregister tray object");
            }
        }

        if let Some(owner_id) = self.owner_id.take() {
            gio::bus_unown_name(owner_id);
        }
    }
}

/// Starts tray integration.
#[cfg(feature = "appindicator-tray")]
pub(crate) fn start(
    accounts: &[Account],
    events: EventBus,
    custom_icon_path: Option<PathBuf>,
) -> Result<Tray> {
    if gtk::is_initialized() && !gtk3::is_initialized() {
        return Err(anyhow!(
            "libappindicator uses GTK3 and cannot be initialized after GTK4"
        ));
    }

    if !gtk3::is_initialized() {
        gtk3::init().context("failed to initialize GTK3 tray menu backend")?;
    }

    let icon = icons::TrayIcon::new(custom_icon_path.as_deref());
    let mut indicator = icon.create_indicator("OpenWhatsapp");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_attention_icon_full(icons::UNREAD_ICON_NAME, "OpenWhatsapp unread");

    let mut menu = menu::build(accounts, events.clone());
    indicator.set_menu(&mut menu);
    menu.show_all();

    let indicator = Rc::new(RefCell::new(indicator));
    bind_unread_badge(Rc::clone(&indicator), events);

    Ok(Tray { indicator, menu })
}

/// Starts tray integration.
#[cfg(not(feature = "appindicator-tray"))]
pub(crate) fn start(
    _accounts: &[Account],
    events: EventBus,
    custom_icon_path: Option<PathBuf>,
) -> Result<Tray> {
    start_status_notifier(events, custom_icon_path)
}

#[cfg(feature = "appindicator-tray")]
fn bind_unread_badge(indicator: Rc<RefCell<AppIndicator>>, events: EventBus) {
    let mut receiver = events.subscribe();

    gtk::glib::MainContext::default().spawn_local(async move {
        let mut unread_by_account: HashMap<Uuid, u32> = HashMap::new();

        loop {
            match receiver.recv().await {
                Ok(AppEvent::UnreadCountChanged { account_id, count }) => {
                    unread_by_account.insert(account_id, count);
                    let total = unread_by_account.values().sum();
                    icons::apply_unread_state(&mut indicator.borrow_mut(), total);
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

#[cfg(not(feature = "appindicator-tray"))]
const SNI_INTERFACE: &str = "org.kde.StatusNotifierItem";
#[cfg(not(feature = "appindicator-tray"))]
const SNI_WATCHER_BUS: &str = "org.kde.StatusNotifierWatcher";
#[cfg(not(feature = "appindicator-tray"))]
const SNI_WATCHER_PATH: &str = "/StatusNotifierWatcher";
#[cfg(not(feature = "appindicator-tray"))]
const SNI_ITEM_PATH: &str = "/StatusNotifierItem";
#[cfg(not(feature = "appindicator-tray"))]
const SNI_MENU_PATH: &str = "/Menu";
#[cfg(not(feature = "appindicator-tray"))]
const DBUS_MENU_INTERFACE: &str = "com.canonical.dbusmenu";
#[cfg(not(feature = "appindicator-tray"))]
const ASSET_TRAY_ICON: &str = "assets/icons/openwhatsapp.png";
#[cfg(not(feature = "appindicator-tray"))]
const TRAY_MENU_REVISION: u32 = 1;
#[cfg(not(feature = "appindicator-tray"))]
const MENU_SHOW_ID: i32 = 1;
#[cfg(not(feature = "appindicator-tray"))]
const MENU_HIDE_ID: i32 = 2;
#[cfg(not(feature = "appindicator-tray"))]
const MENU_QUIT_ID: i32 = 3;

#[cfg(not(feature = "appindicator-tray"))]
const SNI_XML: &str = r#"
<node>
  <interface name="org.kde.StatusNotifierItem">
    <property name="Category" type="s" access="read"/>
    <property name="Id" type="s" access="read"/>
    <property name="Title" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="WindowId" type="u" access="read"/>
    <property name="IconName" type="s" access="read"/>
    <property name="IconThemePath" type="s" access="read"/>
    <property name="IconPixmap" type="a(iiay)" access="read"/>
    <property name="OverlayIconName" type="s" access="read"/>
    <property name="OverlayIconPixmap" type="a(iiay)" access="read"/>
    <property name="AttentionIconName" type="s" access="read"/>
    <property name="AttentionIconPixmap" type="a(iiay)" access="read"/>
    <property name="AttentionMovieName" type="s" access="read"/>
    <property name="ToolTip" type="(sa(iiay)ss)" access="read"/>
    <property name="ItemIsMenu" type="b" access="read"/>
    <property name="Menu" type="o" access="read"/>
    <method name="ContextMenu">
      <arg name="x" type="i" direction="in"/>
      <arg name="y" type="i" direction="in"/>
    </method>
    <method name="Activate">
      <arg name="x" type="i" direction="in"/>
      <arg name="y" type="i" direction="in"/>
    </method>
    <method name="SecondaryActivate">
      <arg name="x" type="i" direction="in"/>
      <arg name="y" type="i" direction="in"/>
    </method>
    <method name="Scroll">
      <arg name="delta" type="i" direction="in"/>
      <arg name="orientation" type="s" direction="in"/>
    </method>
    <signal name="NewTitle"/>
    <signal name="NewIcon"/>
    <signal name="NewAttentionIcon"/>
    <signal name="NewOverlayIcon"/>
    <signal name="NewToolTip"/>
    <signal name="NewStatus">
      <arg name="status" type="s"/>
    </signal>
  </interface>
</node>
"#;

#[cfg(not(feature = "appindicator-tray"))]
const DBUS_MENU_XML: &str = r#"
<node>
  <interface name="com.canonical.dbusmenu">
    <property name="Version" type="u" access="read"/>
    <property name="TextDirection" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="IconThemePath" type="as" access="read"/>
    <method name="GetLayout">
      <arg name="parentId" type="i" direction="in"/>
      <arg name="recursionDepth" type="i" direction="in"/>
      <arg name="propertyNames" type="as" direction="in"/>
      <arg name="revision" type="u" direction="out"/>
      <arg name="layout" type="(ia{sv}av)" direction="out"/>
    </method>
    <method name="GetGroupProperties">
      <arg name="ids" type="ai" direction="in"/>
      <arg name="propertyNames" type="as" direction="in"/>
      <arg name="properties" type="a(ia{sv})" direction="out"/>
    </method>
    <method name="GetProperty">
      <arg name="id" type="i" direction="in"/>
      <arg name="name" type="s" direction="in"/>
      <arg name="value" type="v" direction="out"/>
    </method>
    <method name="Event">
      <arg name="id" type="i" direction="in"/>
      <arg name="eventId" type="s" direction="in"/>
      <arg name="data" type="v" direction="in"/>
      <arg name="timestamp" type="u" direction="in"/>
    </method>
    <method name="AboutToShow">
      <arg name="id" type="i" direction="in"/>
      <arg name="needUpdate" type="b" direction="out"/>
    </method>
    <signal name="LayoutUpdated">
      <arg name="revision" type="u"/>
      <arg name="parent" type="i"/>
    </signal>
  </interface>
</node>
"#;

#[cfg(not(feature = "appindicator-tray"))]
#[derive(Debug)]
struct StatusNotifierState {
    icon_name: String,
    icon_theme_path: String,
    unread_count: u32,
}

#[cfg(not(feature = "appindicator-tray"))]
impl StatusNotifierState {
    fn property(&self, property_name: &str) -> glib::Variant {
        match property_name {
            "Category" => "Communications".to_variant(),
            "Id" => "openwhatsapp".to_variant(),
            "Title" => "OpenWhatsapp".to_variant(),
            "Status" => self.status().to_variant(),
            "WindowId" => 0u32.to_variant(),
            "IconName" => self.current_icon_name().to_variant(),
            "IconThemePath" => self.icon_theme_path.to_variant(),
            "IconPixmap" | "OverlayIconPixmap" | "AttentionIconPixmap" => {
                tray_icon_pixmaps().to_variant()
            }
            "OverlayIconName" | "AttentionMovieName" => String::new().to_variant(),
            "AttentionIconName" => String::new().to_variant(),
            "ToolTip" => self.tooltip(),
            "ItemIsMenu" => false.to_variant(),
            "Menu" => object_path_variant(SNI_MENU_PATH),
            _ => String::new().to_variant(),
        }
    }

    fn current_icon_name(&self) -> &str {
        &self.icon_name
    }

    fn status(&self) -> &'static str {
        if self.unread_count == 0 {
            "Active"
        } else {
            "NeedsAttention"
        }
    }

    fn tooltip(&self) -> glib::Variant {
        glib::Variant::tuple_from_iter([
            "OpenWhatsapp".to_variant(),
            tray_icon_pixmaps().to_variant(),
            "OpenWhatsapp".to_variant(),
            tooltip_body(self.unread_count).to_variant(),
        ])
    }
}

#[cfg(not(feature = "appindicator-tray"))]
fn start_status_notifier(events: EventBus, custom_icon_path: Option<PathBuf>) -> Result<Tray> {
    let connection = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE)
        .context("failed to connect to session bus")?;
    let node = gio::DBusNodeInfo::for_xml(SNI_XML).context("failed to parse tray D-Bus XML")?;
    let interface = node
        .lookup_interface(SNI_INTERFACE)
        .ok_or_else(|| anyhow!("missing tray D-Bus interface"))?;
    let menu_node =
        gio::DBusNodeInfo::for_xml(DBUS_MENU_XML).context("failed to parse tray menu D-Bus XML")?;
    let menu_interface = menu_node
        .lookup_interface(DBUS_MENU_INTERFACE)
        .ok_or_else(|| anyhow!("missing tray menu D-Bus interface"))?;
    let state = Rc::new(RefCell::new(StatusNotifierState {
        icon_name: icon_name(custom_icon_path.as_ref()),
        icon_theme_path: icon_theme_path(custom_icon_path.as_ref()),
        unread_count: 0,
    }));

    let method_events = events.clone();
    let property_state = Rc::clone(&state);
    let registration_id = connection
        .register_object(SNI_ITEM_PATH, &interface)
        .method_call(
            move |_connection,
                  _sender,
                  _object_path,
                  _interface_name,
                  method,
                  _params,
                  invocation| {
                match method {
                    "Activate" | "SecondaryActivate" => {
                        let _subscriber_count = method_events.emit(AppEvent::WindowShow);
                        invocation.return_value(None);
                    }
                    "ContextMenu" | "Scroll" => invocation.return_value(None),
                    _ => invocation.return_dbus_error(
                        "org.freedesktop.DBus.Error.UnknownMethod",
                        "Unknown tray method",
                    ),
                }
            },
        )
        .property(
            move |_connection, _sender, _object_path, _interface_name, property| {
                property_state.borrow().property(property)
            },
        )
        .build()
        .context("failed to register tray D-Bus object")?;
    let menu_registration_id = connection
        .register_object(SNI_MENU_PATH, &menu_interface)
        .method_call({
            let events = events.clone();
            move |_connection,
                  _sender,
                  _object_path,
                  _interface_name,
                  method,
                  params,
                  invocation| {
                handle_menu_method(&events, method, &params, invocation);
            }
        })
        .property(
            move |_connection, _sender, _object_path, _interface_name, property| {
                menu_property(property)
            },
        )
        .build()
        .context("failed to register tray menu D-Bus object")?;

    let service_name = tray_service_name();
    let owner_id = gio::bus_own_name_on_connection(
        &connection,
        &service_name,
        gio::BusNameOwnerFlags::NONE,
        |_connection, _name| {},
        |_connection, _name| {},
    );

    register_status_notifier(&connection)?;
    bind_sni_unread_badge(connection.clone(), state, events);

    Ok(Tray {
        connection,
        registration_ids: vec![registration_id, menu_registration_id],
        owner_id: Some(owner_id),
    })
}

#[cfg(not(feature = "appindicator-tray"))]
fn register_status_notifier(connection: &gio::DBusConnection) -> Result<()> {
    connection
        .call_sync(
            Some(SNI_WATCHER_BUS),
            SNI_WATCHER_PATH,
            SNI_WATCHER_BUS,
            "RegisterStatusNotifierItem",
            Some(&(SNI_ITEM_PATH,).to_variant()),
            None,
            gio::DBusCallFlags::NONE,
            1_000,
            gio::Cancellable::NONE,
        )
        .context("status notifier watcher unavailable")?;

    Ok(())
}

#[cfg(not(feature = "appindicator-tray"))]
fn bind_sni_unread_badge(
    connection: gio::DBusConnection,
    state: Rc<RefCell<StatusNotifierState>>,
    events: EventBus,
) {
    let mut receiver = events.subscribe();

    gtk::glib::MainContext::default().spawn_local(async move {
        let mut unread_by_account: std::collections::HashMap<uuid::Uuid, u32> =
            std::collections::HashMap::new();

        loop {
            match receiver.recv().await {
                Ok(AppEvent::UnreadCountChanged { account_id, count }) => {
                    unread_by_account.insert(account_id, count);
                    let total = unread_by_account.values().sum();
                    state.borrow_mut().unread_count = total;
                    emit_tray_update(&connection, state.borrow().status());
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

#[cfg(not(feature = "appindicator-tray"))]
fn handle_menu_method(
    events: &EventBus,
    method: &str,
    params: &glib::Variant,
    invocation: gio::DBusMethodInvocation,
) {
    match method {
        "GetLayout" => invocation.return_value(Some(&menu_layout_response())),
        "GetGroupProperties" => {
            invocation.return_value(Some(&menu_group_properties().to_variant()))
        }
        "GetProperty" => {
            let id = params.child_get::<i32>(0);
            let name = params.child_get::<String>(1);
            invocation.return_value(Some(&(menu_item_property(id, &name),).to_variant()));
        }
        "Event" => {
            let id = params.child_get::<i32>(0);
            let event_id = params.child_get::<String>(1);
            if event_id == "clicked" {
                emit_menu_event(events, id);
            }
            invocation.return_value(None);
        }
        "AboutToShow" => invocation.return_value(Some(&(false,).to_variant())),
        _ => invocation.return_dbus_error(
            "org.freedesktop.DBus.Error.UnknownMethod",
            "Unknown tray menu method",
        ),
    }
}

#[cfg(not(feature = "appindicator-tray"))]
fn emit_menu_event(events: &EventBus, id: i32) {
    let event = match id {
        MENU_SHOW_ID => Some(AppEvent::WindowShow),
        MENU_HIDE_ID => Some(AppEvent::WindowHide),
        MENU_QUIT_ID => Some(AppEvent::Quit),
        _ => None,
    };

    if let Some(event) = event {
        let _subscriber_count = events.emit(event);
    }
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_property(property: &str) -> glib::Variant {
    match property {
        "Version" => 3u32.to_variant(),
        "TextDirection" => "ltr".to_variant(),
        "Status" => "normal".to_variant(),
        "IconThemePath" => Vec::<String>::new().to_variant(),
        _ => String::new().to_variant(),
    }
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_layout() -> glib::Variant {
    menu_item(
        0,
        "",
        vec![
            menu_item(MENU_SHOW_ID, "Show", Vec::new()),
            menu_item(MENU_HIDE_ID, "Hide", Vec::new()),
            menu_item(MENU_QUIT_ID, "Quit", Vec::new()),
        ],
    )
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_layout_response() -> glib::Variant {
    glib::Variant::tuple_from_iter([TRAY_MENU_REVISION.to_variant(), menu_layout()])
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_item(id: i32, label: &str, children: Vec<glib::Variant>) -> glib::Variant {
    glib::Variant::tuple_from_iter([
        id.to_variant(),
        menu_item_properties(id, label).to_variant(),
        children.to_variant(),
    ])
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_item_properties(id: i32, label: &str) -> BTreeMap<String, glib::Variant> {
    let mut properties = BTreeMap::new();
    properties.insert("enabled".to_string(), true.to_variant());
    properties.insert("visible".to_string(), true.to_variant());
    properties.insert("type".to_string(), String::new().to_variant());

    if id == 0 {
        properties.insert("children-display".to_string(), "submenu".to_variant());
    } else {
        properties.insert("label".to_string(), label.to_variant());
    }

    properties
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_group_properties() -> Vec<(i32, BTreeMap<String, glib::Variant>)> {
    vec![
        (MENU_SHOW_ID, menu_item_properties(MENU_SHOW_ID, "Show")),
        (MENU_HIDE_ID, menu_item_properties(MENU_HIDE_ID, "Hide")),
        (MENU_QUIT_ID, menu_item_properties(MENU_QUIT_ID, "Quit")),
    ]
}

#[cfg(not(feature = "appindicator-tray"))]
fn menu_item_property(id: i32, name: &str) -> glib::Variant {
    let label = match id {
        MENU_SHOW_ID => "Show",
        MENU_HIDE_ID => "Hide",
        MENU_QUIT_ID => "Quit",
        _ => "",
    };

    menu_item_properties(id, label)
        .remove(name)
        .unwrap_or_else(|| String::new().to_variant())
}

#[cfg(not(feature = "appindicator-tray"))]
fn emit_tray_update(connection: &gio::DBusConnection, status: &str) {
    if let Err(error) = connection.emit_signal(
        None,
        SNI_ITEM_PATH,
        SNI_INTERFACE,
        "NewStatus",
        Some(&(status,).to_variant()),
    ) {
        tracing::debug!(?error, "failed to emit tray status update");
    }

    if let Err(error) = connection.emit_signal(None, SNI_ITEM_PATH, SNI_INTERFACE, "NewIcon", None)
    {
        tracing::debug!(?error, "failed to emit tray icon update");
    }
}

#[cfg(not(feature = "appindicator-tray"))]
fn icon_name(custom_icon_path: Option<&PathBuf>) -> String {
    custom_icon_path
        .and_then(|path| path.file_stem())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_default()
}

#[cfg(not(feature = "appindicator-tray"))]
fn icon_theme_path(custom_icon_path: Option<&PathBuf>) -> String {
    custom_icon_path
        .and_then(|path| path.parent())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(not(feature = "appindicator-tray"))]
fn tray_service_name() -> String {
    format!(
        "org.kde.StatusNotifierItem.openwhatsapp_{}_1",
        std::process::id()
    )
}

#[cfg(not(feature = "appindicator-tray"))]
fn tray_icon_pixmaps() -> Vec<(i32, i32, Vec<u8>)> {
    load_icon_pixmaps().unwrap_or_else(generated_tray_pixmaps)
}

#[cfg(not(feature = "appindicator-tray"))]
fn load_icon_pixmaps() -> Option<Vec<(i32, i32, Vec<u8>)>> {
    let sizes = [32, 64];
    let mut pixmaps = Vec::with_capacity(sizes.len());

    for size in sizes {
        let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(ASSET_TRAY_ICON, size, size, true)
            .inspect_err(|error| tracing::debug!(?error, "failed to load tray icon asset"))
            .ok()?;
        pixmaps.push(pixbuf_to_sni_pixmap(&pixbuf)?);
    }

    Some(pixmaps)
}

#[cfg(not(feature = "appindicator-tray"))]
fn pixbuf_to_sni_pixmap(pixbuf: &gdk_pixbuf::Pixbuf) -> Option<(i32, i32, Vec<u8>)> {
    let width = pixbuf.width();
    let height = pixbuf.height();
    let channels = usize::try_from(pixbuf.n_channels()).ok()?;
    let rowstride = usize::try_from(pixbuf.rowstride()).ok()?;
    let width_usize = usize::try_from(width).ok()?;
    let height_usize = usize::try_from(height).ok()?;
    if channels != 3 && channels != 4 {
        return None;
    }

    let pixels = unsafe { pixbuf.pixels() };
    let mut argb = Vec::with_capacity(width_usize * height_usize * 4);
    for y in 0..height_usize {
        for x in 0..width_usize {
            let offset = y * rowstride + x * channels;
            let r = *pixels.get(offset)?;
            let g = *pixels.get(offset + 1)?;
            let b = *pixels.get(offset + 2)?;
            let a = if channels == 4 {
                *pixels.get(offset + 3)?
            } else {
                0xff
            };
            argb.extend_from_slice(&[a, r, g, b]);
        }
    }

    Some((width, height, argb))
}

#[cfg(not(feature = "appindicator-tray"))]
fn generated_tray_pixmaps() -> Vec<(i32, i32, Vec<u8>)> {
    vec![
        (32, 32, generated_tray_argb(32)),
        (64, 64, generated_tray_argb(64)),
    ]
}

#[cfg(not(feature = "appindicator-tray"))]
fn generated_tray_argb(size: i32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    let center = (f64::from(size) - 1.0) / 2.0;
    let radius = f64::from(size) * 0.43;
    let tail_center_x = f64::from(size) * 0.30;
    let tail_center_y = f64::from(size) * 0.76;

    for y in 0..size {
        for x in 0..size {
            let dx = f64::from(x) - center;
            let dy = f64::from(y) - center;
            let in_circle = (dx * dx + dy * dy).sqrt() <= radius;
            let in_tail = f64::from(x) < tail_center_x
                && f64::from(y) > tail_center_y
                && f64::from(y) < f64::from(size) * 0.92
                && f64::from(x) + f64::from(y) > f64::from(size) * 0.92;

            let color = if in_circle || in_tail {
                [0xff, 0x22, 0xc5, 0x5e]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };

            pixels.extend_from_slice(&color);
        }
    }

    pixels
}

#[cfg(not(feature = "appindicator-tray"))]
fn object_path_variant(path: &str) -> glib::Variant {
    match glib::variant::ObjectPath::try_from(path) {
        Ok(object_path) => object_path.to_variant(),
        Err(error) => {
            tracing::debug!(?error, ?path, "invalid tray object path");
            match glib::variant::ObjectPath::try_from("/") {
                Ok(object_path) => object_path.to_variant(),
                Err(_) => String::new().to_variant(),
            }
        }
    }
}

#[cfg(not(feature = "appindicator-tray"))]
fn tooltip_body(unread_count: u32) -> String {
    if unread_count == 0 {
        "WhatsApp Web".to_string()
    } else {
        format!("{unread_count} unread")
    }
}

#[cfg(all(test, not(feature = "appindicator-tray")))]
mod tests {
    use super::*;

    #[test]
    fn tray_service_name_uses_valid_prefix() {
        assert!(tray_service_name().starts_with("org.kde.StatusNotifierItem."));
        assert!(!tray_service_name().contains('-'));
    }

    #[test]
    fn tooltip_mentions_unread_count() {
        assert_eq!(tooltip_body(0), "WhatsApp Web");
        assert_eq!(tooltip_body(7), "7 unread");
    }

    #[test]
    fn tray_icon_pixmap_has_argb_bytes() {
        assert_eq!(generated_tray_argb(32).len(), 32 * 32 * 4);
    }

    #[test]
    fn menu_layout_response_has_expected_type() {
        assert_eq!(menu_layout_response().type_().as_str(), "(u(ia{sv}av))");
    }
}
