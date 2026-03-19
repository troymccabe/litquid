import * as esbuild from 'esbuild';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');
const componentsDir = path.resolve(__dirname, '../../components');
const toolchainDir = path.resolve(__dirname, '../../toolchain');
const distJsDir = path.join(rootDir, 'wwwroot/dist/js');

async function build() {
    fs.mkdirSync(distJsDir, { recursive: true });
    fs.mkdirSync(path.join(distJsDir, 'templates'), { recursive: true });

    // 1. Run the Rust preprocessor to generate templates
    const templatesInput = path.join(componentsDir, 'templates');
    const templatesOutput = path.join(componentsDir, 'scripts/templates');

    // Ensure templates output directory exists
    fs.mkdirSync(templatesOutput, { recursive: true });

    try {
        execSync(`cargo run --bin litquid -- --input "${templatesInput}" --output "${templatesOutput}"`, {
            cwd: toolchainDir,
            stdio: 'inherit',
            shell: true
        });
    } catch (err) {
        console.error('Failed to run litquid preprocessor:', err.message);
    }

    // 2. Build component scripts from shared components directory (templates are bundled in)
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
                minify: true,
            });
            console.log(`Built: ${script.replace(/\.ts$/, '.js')}`);
        }
    }

    console.log('Build complete!');
}

build().catch(err => {
    console.error('Build failed:', err);
    process.exit(1);
});
