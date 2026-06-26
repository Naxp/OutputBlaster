#include "Ghostbusters.h"

static int WindowsLoop()
{
    // Score P1: 0x08E16368 (confirmed, +55 per ghost hit, reached 4395)
    uint32_t scoreP1 = helpers->ReadInt32(0x08E16368, false);

    // Ghosts P1 (lifetime kills): 0x08E16A54 (confirmed, 0->406 across sessions)
    uint32_t ghostsP1 = helpers->ReadInt32(0x08E16A54, false);

    // Credits P1: 0x08E161E8 (confirmed, matches dollar amount)
    uint32_t creditsP1 = helpers->ReadInt32(0x08E161E8, false);

    // Credits P2: 0x08E161EC (confirmed, matches dollar amount)
    uint32_t creditsP2 = helpers->ReadInt32(0x08E161EC, false);

    Outputs->SetValue(OutputHighScore, (UINT8)scoreP1);
    Outputs->SetValue(OutputCoin1, (UINT8)creditsP1);
    Outputs->SetValue(OutputCoin2, (UINT8)creditsP2);
    Outputs->SetValue(OutputAmmo1pA, (UINT8)ghostsP1);

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

void Ghostbusters::OutputsGameLoop()
{
    if (!init)
    {
        AutoLaunchWinGame();
        Outputs = CreateOutputsFromConfig();
        m_game.name = "Ghostbusters";
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
