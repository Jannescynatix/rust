# backend/app.py
from flask import Flask, request, jsonify
from flask_cors import CORS
import joblib

# Flask-App initialisieren
app = Flask(__name__)
CORS(app) # Wichtig für die Kommunikation mit dem Frontend

# Modell und Vektorizer laden
try:
    model = joblib.load('model.pkl')
    vectorizer = joblib.load('tokenizer.pkl')
except FileNotFoundError:
    print("Modell- oder Tokenizer-Datei nicht gefunden. Bitte `train_model.py` ausführen.")
    exit()

@app.route('/predict', methods=['POST'])
def predict():
    data = request.json
    text = data.get('text', '')

    if not text:
        return jsonify({'error': 'Kein Text bereitgestellt.'}), 400

    text_vectorized = vectorizer.transform([text])
    probabilities = model.predict_proba(text_vectorized)[0]
    classes = model.classes_

    # Wahrscheinlichkeiten für "menschlich" und "ki" extrahieren
    mensch_prob = float(probabilities[classes == 'menschlich'])
    ki_prob = float(probabilities[classes == 'ki'])

    return jsonify({
        'menschlich': round(mensch_prob * 100, 2),
        'ki': round(ki_prob * 100, 2)
    })

if __name__ == '__main__':
    app.run(debug=True)