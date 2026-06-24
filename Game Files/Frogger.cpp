#include "Frogger.h"

static int debugCounter = 0;

static int WindowsLoop()
{
    uint32_t tickets = helpers->ReadInt32(0x135C80, true);
    uint32_t coin1 = helpers->ReadInt32(0x0C91F8, true);
    uint32_t coin2 = helpers->ReadInt32(0x0C9230, true);

    Outputs->SetValue(OutputTicketCounter, (UINT8)tickets);
    Outputs->SetValue(OutputCoin1, (UINT8)coin1);
    Outputs->SetValue(OutputCoin2, (UINT8)coin2);
    Outputs->SetValue(OutputHighScore, (UINT8)tickets);

    if (++debugCounter >= 50)
    {
        debugCounter = 0;
        uint32_t lamp1 = helpers->ReadInt32(0x41B86BC, true);
        uint32_t lamp2 = helpers->ReadInt32(0x731970, true);
        uint32_t lamp3 = helpers->ReadInt32(0x135000, true);
        uint32_t lamp4 = helpers->ReadInt32(0x0C9000, true);

        static char msg[256];
        sprintf(msg, "OB Frogger: tix=%u c1=%u c2=%u lamps=0x%08X/0x%08X/0x%08X/0x%08X",
            tickets, coin1, coin2, lamp1, lamp2, lamp3, lamp4);
        OutputDebugStringA(msg);
    }

    return 0;
}

static DWORD WINAPI OutputsAreGo(LPVOID lpParam)
{
    while (true)
    {
        WindowsLoop();
        Sleep(SleepA);
    }
}

void Frogger::OutputsGameLoop()
{
    if (!init)
    {
        AutoLaunchWinGame();
        Outputs = CreateOutputsFromConfig();
        m_game.name = "Frogger";
        Outputs->SetGame(m_game);
        Outputs->Initialize();
        Outputs->Attached();
        CreateThread(NULL, 0, OutputsAreGo, NULL, 0, NULL);
        while (GetMessage(&Msg1, NULL, NULL, 0))
        {
            TranslateMessage(&Msg1);
            DispatchMessage(&Msg1);
        }
        init = true;
    }
}
