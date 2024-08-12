import { PlayerInfo as TPlayerInfo } from "../socket";

type Props = {
    playerInfo?: TPlayerInfo;
};

export default function PlayerInfo({ playerInfo }: Props) {
    if (!playerInfo) {
        return <p className="text-xl">Waiting for player...</p>;
    }

    return (
        <div>
            <p className="text-xl">{playerInfo.name}</p>
        </div>
    );
}
