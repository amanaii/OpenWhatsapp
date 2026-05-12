//! Webview content injection helpers.

/// Creates a content manager with OpenWhatsapp compatibility scripts.
pub(crate) fn content_manager() -> webkit6::UserContentManager {
    let manager = webkit6::UserContentManager::new();
    manager.add_script(&webkit6::UserScript::new(
        chromium_compatibility_script(),
        webkit6::UserContentInjectedFrames::TopFrame,
        webkit6::UserScriptInjectionTime::Start,
        &["https://web.whatsapp.com/*"],
        &[],
    ));
    manager
}

fn chromium_compatibility_script() -> &'static str {
    r#"
(() => {
  const brands = [
    { brand: "Chromium", version: "148" },
    { brand: "Google Chrome", version: "148" },
    { brand: "Not=A?Brand", version: "24" },
  ];

  const metadata = {
    brands,
    mobile: false,
    platform: "Linux",
    getHighEntropyValues: async (hints) => {
      const values = {
        architecture: "x86",
        bitness: "64",
        brands,
        fullVersionList: [
          { brand: "Chromium", version: "148.0.7778.96" },
          { brand: "Google Chrome", version: "148.0.7778.96" },
          { brand: "Not=A?Brand", version: "24.0.0.0" },
        ],
        mobile: false,
        model: "",
        platform: "Linux",
        platformVersion: "",
        uaFullVersion: "148.0.7778.96",
        wow64: false,
      };

      return Object.fromEntries(hints.map((hint) => [hint, values[hint]]));
    },
    toJSON: () => ({ brands, mobile: false, platform: "Linux" }),
  };

  if (!navigator.userAgentData) {
    Object.defineProperty(navigator, "userAgentData", {
      configurable: true,
      enumerable: true,
      get: () => metadata,
    });
  }
})();
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_script_targets_chromium() {
        assert!(chromium_compatibility_script().contains("Google Chrome"));
    }
}
