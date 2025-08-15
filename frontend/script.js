// frontend/script.js

document.addEventListener('DOMContentLoaded', () => {
    const analyzeBtn = document.getElementById('analyze-btn');
    const textInput = document.getElementById('text-input');
    const resultContainer = document.getElementById('result-container');
    const resultText = document.getElementById('result-text');
    const kiBar = document.querySelector('.ki-bar');
    const menschBar = document.querySelector('.mensch-bar');
    const kiLabel = document.getElementById('ki-label');
    const menschLabel = document.getElementById('mensch-label');

    // **WICHTIG:** Ersetzen Sie DIESE URL durch die URL Ihrer gehosteten Render-App
    const API_URL = 'https://ihre-ki-detektor-app.onrender.com/predict';

    analyzeBtn.addEventListener('click', async () => {
        const text = textInput.value.trim();

        if (text.length === 0) {
            alert('Bitte geben Sie einen Text ein, um die Analyse zu starten.');
            return;
        }

        resultContainer.classList.add('hidden');
        analyzeBtn.disabled = true;
        analyzeBtn.textContent = 'Analysiere...';

        try {
            const response = await fetch(API_URL, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text: text })
            });

            const data = await response.json();

            if (response.ok) {
                const kiProb = data.ki;
                const menschProb = data.menschlich;

                // Ergebnis anzeigen
                resultContainer.classList.remove('hidden');

                if (kiProb > menschProb) {
                    resultText.textContent = `Dieser Text wurde wahrscheinlich von einer KI generiert.`;
                } else {
                    resultText.textContent = `Dieser Text wurde wahrscheinlich von einem Menschen geschrieben.`;
                }

                // Fortschrittsbalken aktualisieren
                kiBar.style.width = `${kiProb}%`;
                menschBar.style.width = `${menschProb}%`;
                kiLabel.textContent = `KI: ${kiProb}%`;
                menschLabel.textContent = `Menschlich: ${menschProb}%`;
            } else {
                resultText.textContent = `Fehler: ${data.error}`;
            }

        } catch (error) {
            console.error('Fehler bei der API-Anfrage:', error);
            resultText.textContent = 'Es gab ein Problem bei der Verbindung zur API.';
        } finally {
            analyzeBtn.disabled = false;
            analyzeBtn.textContent = 'Analysieren';
        }
    });
});