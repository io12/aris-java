package edu.rpi.aris.gui.submit;

import edu.rpi.aris.net.GradingStatus;
import edu.rpi.aris.net.NetUtil;
import javafx.scene.control.Button;

import java.util.ArrayList;
import java.util.Comparator;

public class ProofInfo extends AssignmentInfo {

    private final int proofId;
    private final String name;
    private final String createdBy;
    private final long createdOn;
    private final boolean isInstructor;
    private ArrayList<SubmissionInfo> children = new ArrayList<>();
    private GradingStatus gradingStatus = GradingStatus.NONE;
    private String status;
    private String date;
    private Button btn;
    private long timestamp;

    public ProofInfo(int proofId, String name, String createdBy, long createdOn, boolean isInstructor) {
        this.proofId = proofId;
        this.name = name;
        this.createdBy = createdBy;
        this.createdOn = createdOn;
        this.isInstructor = isInstructor;
        btn = new Button("Open Template");
    }

    @Override
    public Object getColumnData(int columnNum) {
        switch (columnNum) {
            case 0:
                return name;
            case 1:
                return status;
            case 2:
                return date;
            case 3:
                return btn;
            default:
                return null;
        }
    }

    @Override
    public void addChild(AssignmentInfo info) {
        if (!(info instanceof SubmissionInfo))
            return;
        children.add((SubmissionInfo) info);
        SubmissionInfo subInfo = children.stream().min(Comparator.comparing(SubmissionInfo::getGradingStatus)).orElse(null);
        gradingStatus = subInfo == null ? GradingStatus.NONE : subInfo.getGradingStatus();
        status = subInfo == null ? NetUtil.STATUS_NO_SUBMISSION : subInfo.getStatus();
        date = subInfo == null ? null : subInfo.getDate();
        timestamp = subInfo == null ? -1 : subInfo.getTimestamp();
    }

    @Override
    public ArrayList<SubmissionInfo> getChildren() {
        return children;
    }

    @Override
    public int compareTo(AssignmentInfo o) {
        if (o instanceof ProofInfo) {
            ProofInfo p = (ProofInfo) o;
            return name.compareTo(p.name);
        } else
            return -1;
    }

    @Override
    public String toString() {
        return name;
    }

    public GradingStatus getGradingStatus() {
        return gradingStatus;
    }

    public long getTimestamp() {
        return timestamp;
    }

    public String getName() {
        return name;
    }

    public String getCreatedBy() {
        return createdBy;
    }

    public long getCreatedOn() {
        return createdOn;
    }

    public int getProofId() {
        return proofId;
    }
}
