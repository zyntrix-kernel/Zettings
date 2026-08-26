#include <QGuiApplication>
#include <QQmlApplicationEngine>

using namespace Qt::Literals::StringLiterals;

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    QQmlApplicationEngine engine;
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(1); },
        Qt::QueuedConnection);
    engine.loadFromModule(u"org.zyntrix.zettings"_s, u"Main"_s);

    return app.exec();
}
