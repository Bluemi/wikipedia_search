#!/usr/bin/env python3


import time

import nltk
import spacy
import blingfire


def main():
    text = 'Mit ca. 14.700 Tempeln und 8 Millionen Anhängern ist die Sōtō-Schule neben der Rinzai-shū und Ōbaku-shū die größte der drei japanischen Hauptrichtungen des Zen. Außerdem gibt es diesen Satz.'
    start_time = time.perf_counter()
    sentences = nltk.tokenize.sent_tokenize(text, language='german')
    end_time = time.perf_counter()
    print('nltk:', end_time - start_time)
    print(sentences)

    nlp = spacy.load('de_core_news_sm')
    start_time = time.perf_counter()
    doc = nlp(text)
    sentences = [sent.text for sent in doc.sents]
    end_time = time.perf_counter()
    print('spacy:', end_time - start_time)
    print(sentences)

    start_time = time.perf_counter()
    sentences = blingfire.text_to_sentences(text).split('\n')
    end_time = time.perf_counter()
    print('blingfire:', end_time - start_time)
    print(sentences)




if __name__ == '__main__':
    main()

