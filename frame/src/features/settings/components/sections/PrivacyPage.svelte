<script lang="ts">
  import * as m from "$shared/i18n/messages";
  import ProtectionControls from "../ProtectionControls.svelte";
  import SettingsGroup from "$shared/ui/SettingsGroup";
  import PreferenceSelect from "../PreferenceSelect.svelte";
  import PreferenceSwitch from "../PreferenceSwitch.svelte";
  import ClearBrowsingData from "../ClearBrowsingData.svelte";
  import SiteExceptions from "../SiteExceptions.svelte";
  import PasskeysGroup from "../PasskeysGroup.svelte";
  import { PrivacyDashboard } from "$features/blocker";
</script>

<ProtectionControls />
<PasskeysGroup />

<SettingsGroup title="Web Privacy & Signals">
  <PreferenceSwitch id="privacy.gpc" preference="privacy.gpc" />
  <PreferenceSwitch id="privacy.dnt" preference="privacy.dnt" />
</SettingsGroup>

<SettingsGroup title={m.settings_site_permissions()}>
  <PreferenceSelect id="privacy.camera" preference="privacy.camera" />
  <PreferenceSelect id="privacy.microphone" preference="privacy.microphone" />
  <PreferenceSelect id="privacy.location" preference="privacy.location" />
  <PreferenceSelect id="privacy.notifications" preference="privacy.notifications" />
</SettingsGroup>

<SettingsGroup title={m.privacy_media_group()}>
  <PreferenceSelect id="privacy.popups" preference="privacy.popups" />
  <PreferenceSelect id="privacy.clipboard" preference="privacy.clipboard" />
  <PreferenceSelect id="privacy.automatic-downloads" preference="privacy.automatic-downloads" />
  <PreferenceSelect id="privacy.sound" preference="privacy.sound" />
</SettingsGroup>

<SiteExceptions />

<SettingsGroup title={m.settings_browsing_data()}>
  <PreferenceSwitch id="privacy.cleanup" preference="privacy.on-exit" />
  <ClearBrowsingData />
</SettingsGroup>

<SettingsGroup title={m.privacy_dashboard()}>
  <PrivacyDashboard />
</SettingsGroup>
