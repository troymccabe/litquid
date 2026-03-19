import * as esbuild from 'esbuild';
import chokidar from 'chokidar';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');
const componentsDir = path.resolve(__dirname, '../../components');
const runtimeDir = path.resolve(__dirname, '../../runtime/src');
const distJsDir = path.join(rootDir, 'wwwroot/dist/js');
const templatesDir = path.join(componentsDir, 'scripts/templates');

async function build() {
    try {
        fs.mkdirSync(distJsDir, { recursive: true });

        // Build component scripts from shared components directory (templates are bundled in)
        const scriptsDir = path.join(componentsDir, 'scripts');
        if (fs.existsSync(scriptsDir)) {
            const scripts = fs.readdirSync(scriptsDir).filter(f => f.endsWith('.ts') || f.endsWith('.js'));
            for (const script of scripts) {
                const outfile = path.join(distJsDir, script.replace(/\.ts$/, '.js'));
                await esbuild.build({
                    entryPoints: [path.join(scriptsDir, script)],
                    outfile,
                    bundle: true,
                    format: 'esm',
                    minify: false,
                });
            }
        }

        console.log('[esbuild] Build complete');
    } catch (err) {
        console.error('[esbuild] Build failed:', err.message);
    }
}

// Initial build
await build();

// Watch for changes in shared component scripts, generated templates, and runtime
const watcher = chokidar.watch([
    path.join(componentsDir, 'scripts/**/*.ts'),
    path.join(componentsDir, 'scripts/**/*.js'),
    path.join(templatesDir, '**/*.js'),
    path.join(runtimeDir, '**/*.ts'),
], {
    persistent: true,
    ignoreInitial: true
});

watcher.on('change', async (filePath) => {
    console.log(`[watch] File changed: ${filePath}`);
    await build();
});

watcher.on('add', async (filePath) => {
    console.log(`[watch] File added: ${filePath}`);
    await build();
});

console.log('[watch] Watching for JS changes...');
