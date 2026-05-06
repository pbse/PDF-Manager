import { writable } from 'svelte/store';

export const isLoading = writable<boolean>(false);
export const statusMessage = writable<string>('');
export const isError = writable<boolean>(false);
export const lastSuccessPath = writable<string | null>(null);

let statusTimeout: any = null;

export function showStatus(message: string, error: boolean = false, path: string | null = null, duration: number = 6000) {
    if (statusTimeout) clearTimeout(statusTimeout);
    
    statusMessage.set(message);
    isError.set(error);
    isLoading.set(false);
    lastSuccessPath.set(path);

    if (duration > 0) {
        statusTimeout = setTimeout(() => {
            statusMessage.set('');
            isError.set(false);
            lastSuccessPath.set(null);
        }, duration);
    }
}

export function startLoading(message: string = 'Processing...') {
    if (statusTimeout) clearTimeout(statusTimeout);
    isLoading.set(true);
    statusMessage.set(message);
    isError.set(false);
    lastSuccessPath.set(null);
}