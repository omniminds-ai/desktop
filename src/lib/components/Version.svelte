<script>
    import {app} from '@tauri-apps/api';

    let appVersion = 'x.x.x'
    app.getVersion().then(version => {
        appVersion = version;
    });
    console.log('Tauri App Version:', appVersion);
</script>

<div class="text-xs text-gray-500">
    <span>v{appVersion}</span>
</div>