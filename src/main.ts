function displayError(message: string | null) {
    const errorElement = document.querySelector('#error-msg') as HTMLElement;
    errorElement.textContent = message ?? '';
}

const processButton = document.querySelector('#process-button') as HTMLButtonElement;
function setLoading(loading: boolean) {
    processButton.disabled = loading;
    processButton.textContent = loading ? 'Processing...' : 'Process';
}

function readFileAsUint8Array(file: File): Promise<Uint8Array> {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
            const binaryString = reader.result as string;
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            resolve(bytes);
        };
        reader.onerror = () => reject(reader.error);
        reader.readAsBinaryString(file);
    });
}


processButton.addEventListener('click', async () => {
    const fileSelector = document.querySelector('#file-input') as HTMLInputElement;

    if (fileSelector.files!.length === 0) {
        displayError('No file selected');
        return;
    }
    if (fileSelector.files!.length > 1) {
        displayError('Please select only one file');
        return;
    }
    setLoading(true);
    displayError(null);

    try {
        const wasmModule = await import('amogus');
        const { convert_image } = wasmModule;

        const input_bytes = await readFileAsUint8Array(fileSelector.files![0]);

        const converted_images = convert_image(input_bytes);

        function showImage(image: Uint8Array, element: string) {
            const elem = document.querySelector(element) as HTMLImageElement;
            const blob = new Blob([image]);
            const url = URL.createObjectURL(blob);
            elem!.src = url;
        }

        showImage(converted_images.preview, '#preview-image');
        showImage(converted_images.full, '#full-image');
    } catch (error: any) {
        displayError(error.toString());
    } finally {
        setLoading(false);
    }
});
