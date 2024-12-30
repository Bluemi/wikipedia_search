import time

from sentence_transformers import SentenceTransformer
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity

bi_model = SentenceTransformer("svalabs/bi-electra-ms-marco-german-uncased")

K = 3 # number of top ranks to retrieve

# specify documents and queries
docs = [
    "Auf Netflix gibt es endlich die neue Staffel meiner Lieblingsserie.",
    "Der Gepard jagt seine Beute.",
    "Wir haben in der Agentur ein neues System für Zeiterfassung.",
    "Mein Arzt sagt, dass mir dabei eher ein Orthopäde helfen könnte.",
    "Einen Impftermin kann mir der Arzt momentan noch nicht anbieten.",
    "Auf Kreta hat meine Tochter mit Muscheln eine schöne Sandburg gebaut.",
    "Das historische Zentrum (centro storico) liegt auf mehr als 100 Inseln in der Lagune von Venedig.",
    "Um in Zukunft sein Vermögen zu schützen, sollte man andere Investmentstrategien in Betracht ziehen.",
    "Die Ära der Dinosaurier wurde vermutlich durch den Einschlag eines gigantischen Meteoriten auf der Erde beendet.",
    "Bei ALDI sind die Bananen gerade im Angebot.",
    "Die Entstehung der Erde ist 4,5 milliarden jahre her.",
    "Finanzwerte treiben DAX um mehr als sechs Prozent nach oben Frankfurt/Main gegeben.",
    "DAX dreht ins Minus. Konjunkturdaten und Gewinnmitnahmen belasten Frankfurt/Main.",
]

queries = [
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

long_text = '''
Anton Bruckner komponierte seine 7. Sinfonie in E-Dur (WAB 107) in den Jahren 1881 bis 1883. Mit ihr gelang Bruckner der internationale Durchbruch als Sinfoniker. Sie ist noch im 21. Jahrhundert seine meistgespielte Sinfonie und gilt als eines seiner bedeutendsten Werke.
Der Einfluss des während der Arbeit an der Sinfonie gestorbenen Richard Wagner, für den der zweite Satz als Trauermusik dient, ist erkennbar an chromatischer Harmonik, zitatähnlichen Anspielungen auf Wagners Werk und der Verwendung von Wagnertuben, zudem ist das Hauptthema des ersten Satzes trotz der für Bruckner typischen Struktur der Kettenbildung aus in sich abgeschlossenen Zellen eine Annäherung an Wagners „unendliche Melodie“.
Die Formen der Sätze folgen weitgehend dem klassischen Muster. Die Anlage von Steigerungen in Wellen mit dem Ziel der Schlussapotheose im Finale wurde in dieser Sinfonie durch eine Umstellung der Glieder in abweichender Tonartendisposition im letzten Satz ergänzt, so dass die Rückleitung zum Hauptthema des Kopfsatzes am Ende der Sinfonie als logische Entwicklung erscheint.
Wegen der ungewöhnlich rasch erfolgten Uraufführung am 30. Dezember 1884 im Leipziger Stadttheater durch das Gewandhausorchester Leipzig unter Arthur Nikisch blieb die Sinfonie von größeren Überarbeitungen verschont. Nach der Aufführung durch den berühmten Wagner-Dirigenten Hermann Levi in München, der die Drucklegung mit Widmung an den König von Bayern, Ludwig II. vermittelte, fand die Sinfonie trotz Verrissen in der Wiener Presse, etwa von Eduard Hanslick, internationale Verbreitung.
Im Zuge des Brucknerkults des Nationalsozialismus erklang die Siebente Sinfonie anlässlich der Meldung von Adolf Hitlers Selbstmord im Radio. Musik aus der Siebenten drückte zudem in Luchino Viscontis Film Senso die Unterdrückung von Reformbestrebungen aus.
An der reichhaltigen Diskografie lässt sich der Wandel der Bruckner-Interpretation von einem freieren Umgang mit der Partitur zu einem monumentalen Stil gleichbleibender Tempi nachvollziehen. In jüngerer Zeit haben Spezialisten für Alte Musik unterschiedliche Akzente gesetzt. 
Auf einen Quartschritt abwärts folgt ein aufsteigender gebrochener E-Dur-Akkord, majestätisch in gewichtig gleichmäßigen Schritten,[13] der länger gehaltene Spitzenton suggeriert einen „weitdimensionierte[n] Raum“.[14] Auf die großen Intervalle folgen dann zwei kleine Sekundschritte, die das „mediantisch bzw. neapolitanisch leuchtende C-Dur der Takte 7/8 herbeiführen“. Quartschritte und harmonische Wechsel erwirkende unerwartete kleine Sekundschritte begegnen auch in späteren Teilen der Komposition, auch die Tonart C-Dur wird in Folge bedeutend.[15] Eine erste Zäsur tritt in der zweiten Hälfte des neunten Melodietaktes auf, mit den folgenden Taktgruppen ergibt sich das Bild einer für Bruckner untypischen asymmetrischen Gliederung. Berücksichtigt man jedoch die Harmonik und ein „Schwer-leicht-Pendel der Takte“, so passt sich die mit Auftakten aufzufassende Melodik in ein „geradzahliges metrisches Gerüst“ ein.[16] Die Ausbreitung der Melodie über zwei Register erinnert an das Konzept der „Mehrstimmigkeit in der Einstimmigkeit“ als wesentliches Prinzip der Instrumentalmusik des 19. Jahrhunderts.[17]
Das komplette erste Thema wird verändert wiederholt. Die Doppelanlage, Paarung als „Aufstellung und Antwort eines Themas“ ist auch als „Original und Mutante [...] ihrem Wesen nach [...] ‚Durchführung‘“.[18] An die Stelle der kammermusikalischen ersten Version tritt über einem Klangfundament mit tiefen Streichern und Blechblasinstrumenten das Thema in „klangflächenartige[r] Unisonoführung“ von Holzbläsern und Violinen „in Art der Mixturen“ auf der Orgel.[7] Nach einer Steigerung mit Höhepunkt leitet ein Epilog zum zweiten Abschnitt über,[19] der in Analogie zum Orgelspiel durch einen deutlichen „Registerwechsel“ markiert wird.[7] 
'''

long_text = long_text.replace('\n', ' ')

# encode documents and queries
start_time = time.perf_counter()
features_docs = bi_model.encode(docs)
end_time = time.perf_counter()
print('documents encoded in', end_time - start_time, 'seconds')


start_time = time.perf_counter()
features_queries = bi_model.encode(queries)
end_time = time.perf_counter()
print('queries encoded in', end_time - start_time, 'seconds')

# compute pairwise cosine similarity scores
sim = cosine_similarity(features_queries, features_docs)

# print results
# for i, query in enumerate(queries):
#     ranks = np.argsort(-sim[i])
#     print("Query:", query)
#     for j, r in enumerate(ranks[:K]):
#         print(f"[{j}: {sim[i, r]: .3f}]", docs[r])
#     print("-"*96)


while False:
    query = input("Enter a query: ")
    if query in ('exit', ''):
        break
    start_time = time.perf_counter()
    query_features = bi_model.encode([query])
    end_time = time.perf_counter()
    print('query encoded in', end_time - start_time, 'seconds')


for l in [100, 200, 300, 400, 500, 750, 1000, 1500, 2000, 3000]:
    short_text = ' '.join(long_text.split(' ')[:l])
    short_text2 = ' '.join(short_text.split(' ')[:-1])
    start_time = time.perf_counter()
    features = bi_model.encode([short_text, short_text2])
    end_time = time.perf_counter()
    print('---------')
    print('l:', l)
    print('long text encoded in', end_time - start_time, 'seconds')
    print('features:', features.shape, features.dtype)
    print('same: ', np.all(np.equal(features[0], features[1])))


print('#########')
print(bi_model.model_card_data)
