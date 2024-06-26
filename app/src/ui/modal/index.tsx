import Modal from "react-modal";

const customStyles = {
  overlay: {
    backgroundColor: "rgba(0, 0, 0, 0)",
  },
  content: {
    top: "50%",
    left: "50%",
    right: "auto",
    bottom: "auto",
    // marginRight: "-50%",
    transform: "translate(-50%, -50%)",
  },
};

Modal.setAppElement("#root");

function MyModal(props: Modal.Props) {
  return (
    <Modal
      {...props}
      shouldCloseOnOverlayClick
      shouldCloseOnEsc
      style={props.style ? props.style : customStyles}
    />
  );
}

export default MyModal;
