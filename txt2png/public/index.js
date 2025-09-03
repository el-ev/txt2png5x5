import wasmInit, { create_image, set_config, set_panic_hook } from './pkg/txt2png5x5.js';
class Txt2PngModule {
    constructor() {
        this.initalized = false;
        this.encoder = new TextEncoder();
    }
    async init() {
        try {
            await wasmInit();
            set_panic_hook();
            this.initalized = true;
            console.log('Module initialized successfully');
        }
        catch (error) {
            console.error('Failed to initialize module:', error);
            throw new Error();
        }
    }
    setConfig(config) {
        if (!this.initalized) {
            throw new Error('Module not initialized. Call init() first.');
        }
        const configStr = JSON.stringify(config);
        set_config(configStr);
    }
    createImage(text) {
        if (!this.initalized) {
            throw new Error('Module not initialized. Call init() first.');
        }
        const encoded = this.encoder.encode(text);
        const imageData = create_image(encoded);
        return new Uint8Array(imageData);
    }
}
document.addEventListener('DOMContentLoaded', async () => {
    const module = new Txt2PngModule();
    await module.init();
    const textEl = document.getElementById('text');
    const fgColorEl = document.getElementById('fg_color');
    const bgColorEl = document.getElementById('bg_color');
    const resultImg = document.getElementById('result_img');
    const downloadBtn = document.getElementById('download_btn');
    const copyBtn = document.getElementById('copy_btn');
    const imageSizeLabel = document.getElementById('image_size_label');
    const generateImage = () => {
        const imgData = module.createImage(textEl.value);
        const blob = new Blob([imgData], { type: 'image/png' });
        const url = URL.createObjectURL(blob);
        resultImg.src = url;
        resultImg.onload = () => {
            imageSizeLabel.textContent = `Image size: ${resultImg.naturalWidth}x${resultImg.naturalHeight}, File size: ${(blob.size / 1024).toFixed(2)}KB`;
        };
    };
    for (const elName of ['column_count', 'column_width', 'char_spacing', 'line_spacing', 'column_spacing', 'scaling']) {
        const el = document.getElementById(elName);
        el.addEventListener('input', () => {
            const config = {};
            config[elName] = parseInt(el.value, 10);
            module.setConfig(config);
            generateImage();
        });
    }
    for (const elMarginEdge of ['top', 'right', 'bottom', 'left']) {
        const thisEl = document.getElementById(`margin_${elMarginEdge}`);
        const elTop = document.getElementById(`margin_top`);
        const elRight = document.getElementById(`margin_right`);
        const elBottom = document.getElementById(`margin_bottom`);
        const elLeft = document.getElementById(`margin_left`);
        thisEl.addEventListener('input', () => {
            const config = { margins: { top: elTop.value ? parseInt(elTop.value, 10) : 0, right: elRight.value ? parseInt(elRight.value, 10) : 0, bottom: elBottom.value ? parseInt(elBottom.value, 10) : 0, left: elLeft.value ? parseInt(elLeft.value, 10) : 0 } };
            module.setConfig(config);
            generateImage();
        });
    }
    fgColorEl.addEventListener('input', () => {
        const fgColor = fgColorEl.value.split(',').map((v) => parseInt(v.trim(), 10));
        if (fgColor.length === 4 && fgColor.every((v) => !isNaN(v) && v >= 0 && v <= 255)) {
            module.setConfig({ fg_color: fgColor });
            generateImage();
        }
        else {
            console.warn('Invalid foreground color input.');
        }
    });
    bgColorEl.addEventListener('input', () => {
        const bgColor = bgColorEl.value.split(',').map((v) => parseInt(v.trim(), 10));
        if (bgColor.length === 4 && bgColor.every((v) => !isNaN(v) && v >= 0 && v <= 255)) {
            module.setConfig({ bg_color: bgColor });
            generateImage();
        }
        else {
            console.warn('Invalid background color input.');
        }
    });
    textEl.addEventListener('input', generateImage);
    downloadBtn.addEventListener('click', () => {
        if (resultImg.src) {
            const link = document.createElement('a');
            link.href = resultImg.src;
            link.download = 'image.png';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        }
    });
    copyBtn.addEventListener('click', async () => {
        if (resultImg.src) {
            try {
                const response = await fetch(resultImg.src);
                const blob = await response.blob();
                await navigator.clipboard.write([
                    new ClipboardItem({
                        [blob.type]: blob
                    })
                ]);
            }
            catch (err) {
                console.error('Failed to copy image: ', err);
            }
        }
    });
    generateImage();
});
//# sourceMappingURL=index.js.map