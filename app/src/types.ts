export type Move = {
  from: string;
  to: string;
  promotion_piece: string | null;
};

export type Color = "white" | "black";
