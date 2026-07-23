#ifndef ENGINE_H
#define ENGINE_H

namespace automotive {
    class IEngine {
    public:
        virtual ~IEngine() = default;
        virtual void start() = 0;
    };

    class V8Engine : public IEngine {
    private:
        int horsepower;
    public:
        V8Engine(int hp);
        void start() override;
        int getHorsepower();
    };
}

#endif
