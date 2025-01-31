import deglib


def get_docs():
    return [
        "Auf Netflix gibt es endlich die neue Staffel meiner Lieblingsserie.",
        "Der Gepard jagt seine Beute.",
        "Wir haben in der Agentur ein neues System für Zeiterfassung.",
        "Mein Arzt sagt, dass mir dabei eher ein Orthopäde helfen könnte.",
        "Einen Impftermin kann mir der Arzt momentan noch nicht anbieten.",
        "Auf Kreta hat meine Tochter mit Muscheln eine schöne Sandburg gebaut.",
        "Das historische Zentrum (centro storico) liegt auf mehr als 100 Inseln in der Lagune von Venedig.",
        "Um in Zukunft sein Vermögen zu schützen, sollte man andere Investmentstrategien in Betracht ziehen.",
        "Die Ära der Dinosaurier wurde vermutlich durch den Einschlag eines Meteoriten auf der Erde beendet.",
        "Bei ALDI sind die Bananen gerade im Angebot.",
        "Die Entstehung der Erde ist 4,5 milliarden jahre her.",
        "Finanzwerte treiben DAX um mehr als sechs Prozent nach oben Frankfurt/Main gegeben.",
        "DAX dreht ins Minus. Konjunkturdaten und Gewinnmitnahmen belasten Frankfurt/Main.",
    ]


def get_queries():
    return [
        "dax steigt",
        "dax sinkt",
        "probleme mit knieschmerzen",
        "software für urlaubsstunden",
        "raubtier auf der jagd",
        "alter der erde",
        "wie alt ist unser planet?",
        "wie kapital sichern",
        "supermarkt lebensmittel reduziert",
        "wodurch ist der tyrannosaurus aussgestorben",
        "serien streamen"
    ]


def show_summary(queries, query_embeddings, docs, doc_embeddings, k=3):
    graph = deglib.builder.build_from_data(doc_embeddings)

    results, dists = graph.search(query_embeddings, 0.2, k)
    print('results.shape')
    print(results.shape)

    for query, indices in zip(queries, results):
        print(query)
        for doc_index in indices:
            doc = docs[doc_index]
            print('  - "{}"'.format(doc))
