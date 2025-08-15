// public/script.js
document.addEventListener('DOMContentLoaded', () => {
    const form = document.getElementById('factorize-form');
    const numberInput = document.getElementById('number-input');
    const submitBtn = document.getElementById('submit-btn');
    const resultContainer = document.getElementById('result-container');
    const resultNumber = document.getElementById('result-number');
    const resultFactors = document.getElementById('result-factors');
    const resultDuration = document.getElementById('result-duration');
    const resultDurationSec = document.getElementById('result-duration-sec');
    const errorMessage = document.getElementById('error-message');
    const progressContainer = document.getElementById('progress-container');
    const progressBar = document.getElementById('progress-bar');
    const progressText = document.getElementById('progress-text');
    const cancelBtn = document.getElementById('cancel-btn');

    const ws = new WebSocket(`ws://${window.location.host}/ws`);

    ws.onopen = () => {
        console.log('WebSocket-Verbindung hergestellt.');
    };

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        switch (data.type) {
            case 'progress':
                const progress = data.progress;
                progressBar.style.width = `${progress}%`;
                progressText.textContent = `${progress}% abgeschlossen`;
                break;
            case 'done':
                progressBar.style.width = '100%';
                progressText.textContent = '100% abgeschlossen';
                resultNumber.textContent = `Zahl: ${data.number}`;
                resultFactors.textContent = `Faktoren: ${data.factors.join(' x ')}`;
                resultDuration.textContent = `Berechnung dauerte: ${data.durationMs} ms`;
                resultDurationSec.textContent = `(${data.durationSec} Sekunden)`;
                resultContainer.classList.remove('hidden');
                submitBtn.disabled = false;
                cancelBtn.classList.add('hidden');
                break;
            case 'error':
                errorMessage.textContent = data.message;
                errorMessage.classList.remove('hidden');
                submitBtn.disabled = false;
                cancelBtn.classList.add('hidden');
                progressContainer.classList.add('hidden');
                break;
            case 'cancelled':
                errorMessage.textContent = data.message;
                errorMessage.classList.remove('hidden');
                submitBtn.disabled = false;
                cancelBtn.classList.add('hidden');
                progressContainer.classList.add('hidden');
                progressBar.style.width = '0%';
                progressText.textContent = '0% abgeschlossen';
                break;
        }
    };

    ws.onclose = () => {
        console.log('WebSocket-Verbindung getrennt. Server neustart erforderlich.');
        errorMessage.textContent = 'Verbindung zum Server verloren. Bitte Seite neu laden.';
        errorMessage.classList.remove('hidden');
        submitBtn.disabled = true;
    };

    form.addEventListener('submit', (e) => {
        e.preventDefault();
        const number = numberInput.value;
        if (!number) {
            errorMessage.textContent = 'Bitte eine Zahl eingeben.';
            errorMessage.classList.remove('hidden');
            return;
        }

        submitBtn.disabled = true;
        cancelBtn.classList.remove('hidden');
        resultContainer.classList.add('hidden');
        errorMessage.classList.add('hidden');
        progressContainer.classList.remove('hidden');
        progressBar.style.width = '0%';
        progressText.textContent = '0% abgeschlossen';

        ws.send(JSON.stringify({ type: 'startCalculation', number: parseInt(number) }));
    });

    cancelBtn.addEventListener('click', () => {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'cancelCalculation' }));
        }
    });
});